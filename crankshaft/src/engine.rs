//!  Engine.

pub mod config;
pub mod service;
pub mod task;

use std::time::Duration;

use futures::StreamExt;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
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
    pub async fn run(&mut self) {
        let task_completion_bar = ProgressBar::new(self.runner.tasks.len() as u64);
        task_completion_bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan/blue} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        let mut count = 1;
        task_completion_bar.inc(0);
        task_completion_bar.enable_steady_tick(Duration::from_millis(100));

        while let Some(()) = self.runner.tasks.next().await {
            task_completion_bar.set_message(format!("task #{}", count));
            task_completion_bar.inc(1);
            count += 1;
        }

        task_completion_bar.finish_with_message("All jobs complete.");
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::with_docker().expect("could not initialize engine")
    }
}
