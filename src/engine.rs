//!  Engine.

pub mod service;
pub mod task;

use futures::future::join_all;
pub use task::Task;
use tracing::debug;

use crate::engine::service::runner::Handle;
use crate::engine::service::runner::Runner;

/// An engine.
#[derive(Debug)]
pub struct Engine {
    /// The task runner.
    runner: Runner,
}

impl Engine {
    /// Submits a [`Task`] to be executed.
    ///
    /// A [`Handle`] is returned, which contains a channel that can be awaited
    /// for the result of the job.
    pub fn submit(&mut self, task: Task) -> Handle {
        debug!(task = ?task);
        self.runner.submit(task)
    }

    /// Runs all of the tasks scheduled in the engine.
    pub async fn run(self) {
        join_all(self.runner.tasks).await;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            runner: Runner::docker(),
        }
    }
}
