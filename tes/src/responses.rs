use serde::Deserialize;
use serde::Serialize;

/// A response from `POST /tasks`.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTask {
    /// The ID of the created task.
    pub id: String,
}
