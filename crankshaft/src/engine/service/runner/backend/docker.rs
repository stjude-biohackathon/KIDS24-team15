//! A docker runner service.

use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::KillContainerOptions;
use bollard::container::LogOutput;
use bollard::container::StartContainerOptions;
use bollard::container::UploadToContainerOptions;
use bollard::errors::Error;
use bollard::exec::CreateExecOptions;
use bollard::exec::StartExecResults;
use bollard::models::HostConfig;
use bollard::models::Mount;
use bollard::Docker;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::TryStreamExt;
use nonempty::NonEmpty;
use random_word::Lang;
use tmp_mount::TmpMount;
use tokio::sync::oneshot::Sender;

use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::backend::ExecutionResult;
use crate::engine::service::runner::backend::Reply;
use crate::engine::task::Execution;
use crate::engine::task::Input;
use crate::engine::task::Resources;
use crate::engine::Task;

pub mod tmp_mount;

/// The number of parts in the random name of each Docker container.
pub const NAME_PARTS: usize = 4;

/// The separator between each random part of the Docker container name.
pub const NAME_SEPARATOR: &str = "-";

/// The working dir name inside the docker container
pub const WORKDIR: &str = "/workdir";

/// A [`Result`](std::result::Result) with an [`Error`]
pub type Result<T> = std::result::Result<T, Error>;

/// A local execution backend.
#[derive(Debug)]
pub struct DockerBackend {
    /// A handle to the inner docker client.
    client: Arc<Docker>,

    /// Whether or not to clean up containers.
    cleanup: bool,
}

impl DockerBackend {
    /// Attempts to create a new [`Docker`].
    ///
    /// Note that, currently, we connect [using defaults](Docker::connect_with_defaults).
    pub fn try_new(cleanup: bool) -> Result<Self> {
        let inner = Docker::connect_with_defaults().map(Arc::new)?;
        Ok(Self {
            client: inner,
            cleanup,
        })
    }
}

#[async_trait]
impl Backend for DockerBackend {
    fn default_name(&self) -> &'static str {
        "docker"
    }

    fn run(&self, name: String, task: Task, cb: Sender<Reply>) -> BoxFuture<'static, ()> {
        let mut client = self.client.clone();
        let cleanup = self.cleanup;

        async move {
            let mut results: Option<NonEmpty<ExecutionResult>> = None;

            // Generate mounts to be shared among tasks
            let tmp_mounts: Vec<TmpMount> = task
                .volumes()
                .into_iter()
                .flatten()
                .map(|s| TmpMount::from_str(s).unwrap())
                .collect();

            let mounts: Vec<Mount> = tmp_mounts.iter().map(|tm| tm.into()).collect();

            for execution in task.executions() {
                let name = random_name();

                // Create the container
                container_create(&name, execution, task.resources(), &mut client, &mounts[..])
                    .await;

                // Start the container
                container_start(&name, &mut client).await;

                // Insert inputs
                if let Some(inputs) = task.inputs() {
                    for input in inputs {
                        insert_input(&name, &mut client, input).await;
                    }
                };

                // Run a command
                let exec_result = container_exec(&name, execution, &mut client).await;

                if cleanup {
                    client
                        .kill_container(&name, None::<KillContainerOptions<String>>)
                        .await
                        .unwrap();
                    client.remove_container(&name, None).await.unwrap();
                }

                results = match results {
                    Some(mut results) => {
                        results.push(exec_result);
                        Some(results)
                    }
                    None => Some(NonEmpty::new(exec_result)),
                }
            }

            // NOTE: this will return an error if the receiver has already hung
            // up or has been deallocated. In those cases, it simply means the
            // client wasn't interested in the response, so we don't care about
            // this error.
            let _ = cb.send(Reply {
                backend: name,
                executions: Some(results.expect("at least one execution to be run")),
            });
        }
        .boxed()
    }
}

/// Generates a random name for a Docker container.
fn random_name() -> String {
    (1..=NAME_PARTS)
        .map(|_| random_word::r#gen(Lang::En))
        .collect::<Vec<_>>()
        .join(NAME_SEPARATOR)
}

/// Creates a container using the Docker client.
async fn container_create(
    name: &str,
    execution: &Execution,
    resources: Option<&Resources>,
    client: &mut Arc<Docker>,
    mounts: &[Mount],
) {
    // Configure Docker to use all mounts
    let host_config = HostConfig {
        mounts: Some(mounts.to_vec()),
        ..resources.map(HostConfig::from).unwrap_or_default()
    };

    let options = Some(CreateContainerOptions {
        name,
        ..Default::default()
    });

    let config = Config {
        image: Some(execution.image()),
        tty: Some(true),
        host_config: Some(host_config),
        working_dir: execution.workdir().map(String::as_str),
        ..Default::default()
    };

    client.create_container(options, config).await.unwrap();
}

/// Starts a container using the Docker client.
async fn container_start(name: &str, client: &mut Arc<Docker>) {
    client
        .start_container(name, None::<StartContainerOptions<String>>)
        .await
        .unwrap();
}

/// Puts input files into the container
async fn insert_input(name: &str, client: &mut Arc<Docker>, input: &Input) {
    let mut tar = tar::Builder::new(Vec::new());

    let content = input.fetch().await.unwrap();

    let tar_path = input.path().trim_start_matches('/');

    // Create a header with the full path
    let mut header = tar::Header::new_gnu();
    header.set_path(tar_path).unwrap();
    header.set_size(content.len() as u64);
    header.set_mode(0o644); // Set appropriate permissions
    header.set_cksum();

    // Append the file to the tar archive
    tar.append_data(&mut header, tar_path, Cursor::new(content))
        .unwrap();

    let tar_contents = tar.into_inner().unwrap();

    // Upload to the root of the container
    client
        .upload_to_container(
            name,
            Some(UploadToContainerOptions {
                path: "/",
                ..Default::default()
            }),
            tar_contents.into(),
        )
        .await
        .unwrap();
}

/// Execute a command in container, returning an ExecutionResult
async fn container_exec(
    name: &str,
    execution: &Execution,
    client: &mut Arc<Docker>,
) -> ExecutionResult {
    let exec_id = client
        .create_exec(
            name,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(execution.args().into_iter().map(|s| s.as_str()).collect()),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    let log_stream = if let StartExecResults::Attached { output, .. } =
        client.start_exec(&exec_id, None).await.unwrap()
    {
        output
    } else {
        unreachable!();
    };

    // Process logs
    let (stdout, stderr) = log_stream
        .try_fold(
            (String::with_capacity(1 << 8), String::with_capacity(1 << 8)),
            |(mut stdout, mut stderr), log| async move {
                match log {
                    LogOutput::StdOut { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    LogOutput::StdErr { message } => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
                Ok((stdout, stderr))
            },
        )
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error collecting logs: {:?}", e);
            (String::new(), String::new())
        });

    // Get return code
    // Get the exit code
    let exec_inspect = client.inspect_exec(&exec_id).await.unwrap();
    let status = exec_inspect.exit_code.unwrap_or(-1) as u64;

    ExecutionResult {
        status,
        stdout,
        stderr,
    }
}
