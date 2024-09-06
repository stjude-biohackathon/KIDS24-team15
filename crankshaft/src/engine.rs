//!  Engine.

use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use crate::engine::service::runner::backend::docker;
use crate::engine::service::runner::backend::docker::DockerBackend;
use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::Handle;
use crate::engine::service::runner::Runner;

pub mod config;
pub mod service;
pub mod task;

pub use task::Task;

/// The runners stored within the engine.
type Runners = IndexMap<String, Runner>;

/// An engine.
#[derive(Debug)]
pub struct Engine {
    /// The task runner(s).
    runners: Runners,
}

impl Engine {
    /// Creates an empty engine.
    pub fn empty() -> Self {
        Self {
            runners: Default::default(),
        }
    }

    /// Adds a [`Backend`] to the engine.
    pub fn with_backend(mut self, name: impl Into<String>, backend: impl Backend) -> Self {
        let name = name.into();
        self.runners
            .insert(name.clone(), Runner::new(name, backend));
        self
    }

    /// Gets a new engine with a backend.
    pub fn new_with_backend(name: impl Into<String>, backend: impl Backend) -> Self {
        Self::empty().with_backend(name, backend)
    }

    /// Adds a docker backend to a [`Engine`].
    pub fn with_docker(self, cleanup: bool) -> docker::Result<Self> {
        let backend = DockerBackend::try_new(cleanup)?;
        Ok(self.with_backend(backend.default_name(), backend))
    }

    /// Gets a new engine with a default Docker backend.
    pub fn new_with_docker(cleanup: bool) -> docker::Result<Self> {
        Ok(Self::empty()
            .with_docker(cleanup)
            .expect("docker client to connect"))
    }

    /// Gets the names of the runners.
    pub fn runners(&self) -> impl Iterator<Item = &str> {
        self.runners.keys().map(|key| key.as_ref())
    }

    /// Submits a [`Task`] to be executed.
    ///
    /// A [`Handle`] is returned, which contains a channel that can be awaited
    /// for the result of the job.
    pub fn submit(&mut self, name: impl AsRef<str>, task: Task) -> Handle {
        let name = name.as_ref();

        let backend = self
            .runners
            .get(name)
            .unwrap_or_else(|| panic!("backend not found: {name}"));

        backend.submit(task)
    }

    /// Runs all of the tasks scheduled in the engine.
    pub async fn run(self) {
        let mut futures = FuturesUnordered::new();

        for (_, runner) in self.runners {
            futures.extend(runner.tasks());
        }

        let task_completion_bar = ProgressBar::new(futures.len() as u64);
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

        while let Some(()) = futures.next().await {
            task_completion_bar.set_message(format!("task #{}", count));
            task_completion_bar.inc(1);
            count += 1;
        }

        task_completion_bar.finish_with_message("All jobs complete.");
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::empty().with_docker(true).unwrap()
    }
}
