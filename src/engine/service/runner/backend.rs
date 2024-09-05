//! Supported backends.

pub mod docker;
use async_trait::async_trait;
pub use docker::Runner;
use futures::future::BoxFuture;
use nonempty::NonEmpty;
use tokio::sync::oneshot::Sender;

pub mod backend_config;
pub mod generic;

pub use std::fmt::Debug;

use crate::engine::Task;
use crate::BoxedError;

/// A [`Result`](std::result::Result) with a [`BoxedError`]
pub type Result<T> = std::result::Result<T, BoxedError>;

/// A result of a single execution.
#[derive(Debug)]
pub struct ExecutionResult {
    /// The exit code.
    pub status: i64,

    /// The contents of standard out.
    pub stdout: String,

    /// The contents of standard error.
    pub stderr: String,
}

/// A reply from a backend when a task is completed.
#[derive(Debug)]
pub struct Reply {
    /// The results from each execution.
    pub executions: NonEmpty<ExecutionResult>,
}

/// An execution backend.
#[async_trait]
pub trait Backend: Debug + Send + 'static {
    /// Runs a task in a backend;
    fn run(&self, task: Task, cb: Sender<Reply>) -> BoxFuture<'static, ()>;
}
