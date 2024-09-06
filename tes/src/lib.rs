//! Task execution service.

use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use reqwest_middleware::Error;

pub mod responses;
pub mod task;

pub use task::Task;

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// A task execution service (TES) client.
#[derive(Debug)]
pub struct Client {
    /// The base URL.
    url: String,

    /// The client.
    client: reqwest_middleware::ClientWithMiddleware,
}

impl Client {
    /// Creates a new [`Client`].
    pub fn new(url: impl Into<String>, headers: impl Into<HeaderMap>) -> Result<Self> {
        let url = url.into();
        let headers = headers.into();

        let reqwest_client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        let retry_policy = reqwest_retry::policies::ExponentialBackoff::builder().build_with_max_retries(3);
        let client = reqwest_middleware::ClientBuilder::new(reqwest_client)
            .with(reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self { url, client })
    }

    /// Returns whether the URL is healthy by hitting the `GET /service-info`
    /// endpoint.
    pub async fn healthcheck(&self) -> bool {
        let url = format!("{}service-info", self.url);

        self.client
            .get(&url)
            .send()
            .await
            .map(|res| res.status() == StatusCode::OK)
            .unwrap_or(false)
    }

    /// Attempts to create a task.
    pub async fn create_task(&self, task: Task) -> Result<String> {
        let url = format!("{}tasks", self.url);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );

        let json = serde_json::to_string(&task).unwrap();

        let res = self
            .client
            .post(&url)
            .body(json)
            .headers(headers.clone())
            .send()
            .await?;

        let text = &res.text().await?;
        Ok(serde_json::from_str::<responses::CreateTask>(text)
            .unwrap()
            .id)
    }

    /// Gets a task.
    pub async fn get_task(&self, id: &str) -> Result<Task> {
        let url = format!("{}tasks/{}?view=FULL", self.url, id);
        let res = self.client.get(&url).send().await?;
        let task: Task = serde_json::from_str(&res.text().await?).unwrap();
        Ok(task)
    }
}
