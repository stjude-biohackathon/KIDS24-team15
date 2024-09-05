//! Task execution service.

pub mod task;

use reqwest::{
    header::{self},
    Client, Error, StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum FileType {
    #[serde(rename = "FILE")]
    #[default]
    File,

    #[serde(rename = "DIRECTORY")]
    Directory,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub path: String,

    #[serde(rename = "type")]
    pub input_type: FileType,

    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesOutput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub path: String,

    #[serde(rename = "type")]
    pub output_type: FileType,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesResources {
    pub cpu_cores: Option<i64>,
    pub preemptible: Option<bool>,
    pub ram_gb: Option<f64>,
    pub disk_gb: Option<f64>,
    pub zones: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesExecutor {
    pub image: String,
    pub command: Vec<String>,
    pub workdir: Option<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesExecutorLog {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: i32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesOutputFileLog {
    pub url: String,
    pub path: String,
    pub size_bytes: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesTaskLog {
    pub logs: Vec<TesExecutorLog>,
    // metadata???
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub outputs: Vec<TesOutputFileLog>,
    pub system_logs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TesTask {
    pub id: Option<String>,
    pub state: Option<task::State>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub inputs: Option<Vec<TesInput>>,
    pub outputs: Option<Vec<TesOutput>>,
    pub resources: Option<TesResources>,
    pub executors: Vec<TesExecutor>,
    pub volumes: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TesCreateTaskResponse {
    pub id: String,
}

#[derive(Debug)]
pub struct TesHttpClient {
    base_url: String,
    pub client: reqwest::Client,
}

impl TesHttpClient {
    pub fn new(base_url: impl Into<String>, headers: header::HeaderMap) -> Result<Self, Error> {
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            base_url: base_url.into(),
            client,
        })
    }

    pub async fn service_info(&self) -> Result<bool, Error> {
        let url = format!("{}service-info", self.base_url);
        let res = self.client.get(&url).send().await?;
        Ok(res.status() == StatusCode::OK)
    }

    pub async fn create_task(&self, task: TesTask) -> Result<String, Error> {
        let url = format!("{}tasks", self.base_url);

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
        dbg!(text);

        let create_task_response: TesCreateTaskResponse = serde_json::from_str(text).unwrap();
        Ok(create_task_response.id)
    }

    pub async fn get_task(&self, id: &str) -> Result<TesTask, Error> {
        let url = format!("{}tasks/{}?view=FULL", self.base_url, id);
        let res = self.client.get(&url).send().await?;
        let task: TesTask = serde_json::from_str(&res.text().await?).unwrap();
        Ok(task)
    }
}
