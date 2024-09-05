//! A docker runner service.

use std::sync::Arc;

use async_trait::async_trait;
use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::LogOutput;
use bollard::container::LogsOptions;
use bollard::container::StartContainerOptions;
use bollard::container::WaitContainerOptions;
use bollard::errors::Error;
use bollard::secret::ContainerWaitResponse;
use bollard::Docker;
use futures::future::BoxFuture;
use futures::select;
use futures::FutureExt;
use futures::StreamExt;
use nonempty::NonEmpty;
use random_word::Lang;
use tokio::sync::oneshot::Sender;

use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::backend::ExecutionResult;
use crate::engine::service::runner::backend::Reply;
use crate::engine::task::Execution;
use crate::engine::Task;

/// The number of parts in the random name of each Docker container.
pub const NAME_PARTS: usize = 4;

/// The separator between each random part of the Docker container name.
pub const NAME_SEPARATOR: &str = "-";

/// A [`Result`](std::result::Result) with an [`Error`]
pub type Result<T> = std::result::Result<T, Error>;

/// A local execution backend.
#[derive(Debug)]
pub struct Runner {
    /// A handle to the inner docker client.
    client: Arc<Docker>,
}

impl Runner {
    /// Attempts to create a new [`Docker`].
    ///
    /// Note that, currently, we connect [using defaults](Docker::connect_with_defaults).
    pub fn try_new() -> Result<Self> {
        let inner = Docker::connect_with_defaults().map(Arc::new)?;
        Ok(Self { client: inner })
    }
}

#[async_trait]
impl Backend for Runner {
    fn run(&self, task: Task, cb: Sender<Reply>) -> BoxFuture<'static, ()> {
        let mut client = self.client.clone();

        async move {
            let mut results: Option<NonEmpty<ExecutionResult>> = None;

            for execution in task.executions() {
                let name = random_name();

                container_create(&name, execution, &mut client).await;
                container_start(&name, &mut client).await;

                let mut logs = configure_logs(&name, execution, &mut client);
                let mut wait = configure_wait(&name, &mut client);

                let status;
                let mut stdout = String::with_capacity(1 >> 8);
                let mut stderr = String::with_capacity(1 >> 8);

                loop {
                    select! {
                        result = logs.next().fuse() => match result {
                            Some(Ok(LogOutput::StdOut { message })) => {
                                stdout.push_str(&String::from_utf8_lossy(message.as_ref()))
                            }
                            Some(Ok(LogOutput::StdErr { message })) => {
                                stderr.push_str(&String::from_utf8_lossy(message.as_ref()))
                            }
                            Some(Err(e)) => eprintln!("error reading log: {:?}", e),
                            Some(Ok(_)) | None => {}
                        },
                        result = wait.next().fuse() => match result {
                            Some(Ok(response)) => {
                                status = response.status_code;
                                break;
                            }
                            Some(Err(e)) => eprintln!("error waiting for container: {e:?}"),
                            None => unreachable!(),
                        }
                    }
                }

                client.remove_container(&name, None).await.unwrap();

                let result = ExecutionResult {
                    status,
                    stdout,
                    stderr,
                };

                results = match results {
                    Some(mut results) => {
                        results.push(result);
                        Some(results)
                    }
                    None => Some(NonEmpty::new(result)),
                }
            }

            // NOTE: this will return an error if the receiver has already hung
            // up or has been deallocated. In those cases, it simply means the
            // client wasn't interested in the response, so we don't care about
            // this error.
            let _ = cb.send(Reply {
                executions: results.expect("at least one execution to be run"),
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
async fn container_create(name: &str, execution: &Execution, client: &mut Arc<Docker>) {
    let options = Some(CreateContainerOptions {
        name,
        ..Default::default()
    });

    let config = Config {
        image: Some(execution.image()),
        cmd: Some(execution.args().into_iter().map(|s| s.as_str()).collect()),
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

/// Configures the log stream for the Docker container.
fn configure_logs(
    name: &str,
    execution: &Execution,
    client: &mut Arc<Docker>,
) -> impl futures::Stream<Item = std::result::Result<LogOutput, Error>> {
    let options = LogsOptions::<String> {
        follow: true,
        stdout: execution.stdout().is_some(),
        stderr: execution.stderr().is_some(),
        ..Default::default()
    };

    client.logs(name, Some(options))
}

/// Configures the waiting stream for the Docker container.
fn configure_wait(
    name: &str,
    client: &mut Arc<Docker>,
) -> impl futures::Stream<Item = std::result::Result<ContainerWaitResponse, Error>> {
    client.wait_container(name, None::<WaitContainerOptions<String>>)
}
