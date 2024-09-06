//! A task execution service (TES) runner.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::FutureExt as _;
use nonempty::NonEmpty;
use reqwest::header;
use tes::Client;
use tokio::sync::oneshot::Sender;

use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::backend::ExecutionResult;
use crate::engine::service::runner::backend::Reply;
use crate::engine::Task;
use crate::BoxedError;

/// The number of parts in the random name of each TES container.
pub const NAME_PARTS: usize = 4;

/// The separator between each random part of the TES container name.
pub const NAME_SEPARATOR: &str = "-";

/// A [`Result`](std::result::Result) with an [`BoxedError`]
pub type Result<T> = std::result::Result<T, BoxedError>;

/// A local execution backend.
#[derive(Debug)]
pub struct TesBackend {
    /// A handle to the inner TES client.
    client: Arc<Client>,
}

impl TesBackend {
    /// Creates a new [`TesBackend`].
    pub fn new(url: impl Into<String>, token: Option<impl Into<String>>) -> Self {
        let url = url.into();

        let mut headers = header::HeaderMap::new();

        if let Some(token) = token {
            headers.insert(
                "Authorization",
                header::HeaderValue::from_str(&format!("Basic {}", token.into())).unwrap(),
            );
        }

        let inner = Client::new(&url, headers).unwrap();

        Self {
            client: Arc::new(inner),
        }
    }
}

#[async_trait]
impl Backend for TesBackend {
    fn default_name(&self) -> &'static str {
        unimplemented!("you must provide a backend name for a TES runner!")
    }

    fn run(&self, name: String, task: Task, cb: Sender<Reply>) -> BoxFuture<'static, ()> {
        let client = self.client.clone();

        let task = tes::Task {
            name: task.name().map(|v| v.to_owned()),
            description: task.description().map(|v| v.to_owned()),
            executors: task
                .executions()
                .map(|execution| tes::task::Executor {
                    image: execution.image().to_owned(),
                    command: execution.args().into_iter().cloned().collect::<Vec<_>>(),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
            ..Default::default()
        };

        async move {
            let task_id = client.create_task(task).await.unwrap();

            loop {
                if let Ok(task) = client.get_task(&task_id).await {
                    if let Some(ref state) = task.state {
                        if !state.is_executing() {
                            let mut results = task
                                .logs
                                .unwrap()
                                .into_iter()
                                .flat_map(|task| task.logs)
                                .map(|log| ExecutionResult {
                                    status: log.exit_code.unwrap_or_default() as u64,
                                    stdout: log.stdout.unwrap_or_default(),
                                    stderr: log.stderr.unwrap_or_default(),
                                });

                            let mut executions = NonEmpty::new(results.next().unwrap());
                            executions.extend(results);

                            let reply = Reply {
                                backend: name,
                                executions: Some(executions),
                            };

                            let _ = cb.send(reply);
                            return;
                        }

                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        }
        .boxed()
    }
}
