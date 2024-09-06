//!  Engine.

pub mod config;
pub mod service;
pub mod task;

use futures::future::join_all;
pub use task::Task;
use tracing::debug;

use crate::engine::service::runner::backend::docker;
use crate::engine::service::runner::backend::tes;
use crate::engine::service::runner::Handle;
use crate::engine::service::runner::Runner;

/// An engine.
#[derive(Debug)]
pub struct Engine {
    /// The task runner.
    runner: Runner,
}

impl Engine {
    /// Gets an engine with a generic [`Runner`].
    pub fn with_runner(runner: Runner) -> Self {
        Self { runner }
    }

    /// Gets an engine with a Docker backend.
    pub fn with_docker() -> docker::Result<Self> {
        let docker = docker::Runner::try_new()?;
        Ok(Self::with_runner(Runner::new(docker)))
    }

    /// Gets an engine with a default TES backend.
    pub fn with_default_tes() -> Self {
        Self::with_runner(Runner::new(tes::Tes::default()))
    }

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
        Self::with_docker().expect("could not initialize engine")
    }
}
