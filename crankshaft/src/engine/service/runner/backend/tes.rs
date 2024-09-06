//! A task execution service (TES) runner.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::FutureExt as _;
use rand::thread_rng;
use rand::Rng;
use reqwest::header;
use tes::Client;
use tokio::sync::oneshot::Sender;

use crate::engine::service::runner::backend::Backend;
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
pub struct Tes {
    /// A handle to the inner TES client.
    client: Arc<Client>,
    /// Time to sleep in between status checks
    sleep_time: u64,
}

impl Tes {
    /// Attempts to create a new [`Tes`].
    ///
    /// Note that, currently, we connect [using defaults](Docker::connect_with_defaults).
    pub fn try_new(url: impl Into<String>) -> Result<Self> {
        let url = url.into();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Pinggy-No-Screen",
            header::HeaderValue::from_static("value"),
        );

        let inner = Client::new(&url, headers).unwrap();

        let mut rng = thread_rng();
        let sleep_time = rng.gen_range(100..300);

        Ok(Self {
            client: Arc::new(inner),
            sleep_time,
        })
    }
}

impl Default for Tes {
    fn default() -> Self {
        Self::try_new("http://localhost:8080/ga4gh/tes/v1/").unwrap()
    }
}

#[async_trait]
impl Backend for Tes {
    fn run(&self, _: Task, cb: Sender<Reply>) -> BoxFuture<'static, ()> {
        let client = self.client.clone();

        let task = tes::Task {
            name: Some("Hello World".to_string()),
            description: Some("Hello World, inspired by Funnel's most basic example".to_string()),
            executors: vec![tes::task::Executor {
                image: "alpine".to_string(),
                command: vec!["echo".to_string(), "TESK says: Hello World".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let sleep = self.sleep_time;
        async move {
            let task_id = client.create_task(task).await.unwrap();

            loop {
                if let Ok(result) = client.get_task(&task_id).await {
                    if let Some(ref state) = result.state {
                        if !state.is_executing() {
                            break;
                        }

                        tokio::time::sleep(Duration::from_millis(sleep)).await;
                    }
                }
            }

            let reply = Reply { executions: None };

            let _ = cb.send(reply);
        }
        .boxed()
    }
}
