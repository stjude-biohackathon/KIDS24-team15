use serde::Deserialize;
use serde::Serialize;

/// State of TES task.
#[derive(Serialize, Deserialize, Debug, Default)]
pub enum State {
    /// An unknown state.
    #[serde(rename = "UNKNOWN")]
    #[default]
    Unknown,

    /// A queued task.
    #[serde(rename = "QUEUED")]
    Queued,

    /// A task that is initializing.
    #[serde(rename = "INITIALIZING")]
    Initializing,

    /// A task that is running.
    #[serde(rename = "RUNNING")]
    Running,

    /// A task that is paused.
    #[serde(rename = "PAUSED")]
    Paused,

    /// A task that has completed.
    #[serde(rename = "COMPLETE")]
    Complete,

    /// A task that has errored during execution.
    #[serde(rename = "EXECUTOR_ERROR")]
    ExecutorError,

    /// A task that has encountered a system error.
    #[serde(rename = "SYSTEM_ERROR")]
    SystemError,

    /// A task that has been cancelled.
    #[serde(rename = "CANCELED")]
    Canceled,
}

impl State {
    /// Returns whether a task is still executing or not.
    pub fn is_executing(&self) -> bool {
        matches!(
            self,
            Self::Unknown | Self::Queued | Self::Initializing | Self::Running | Self::Paused
        )
    }
}
