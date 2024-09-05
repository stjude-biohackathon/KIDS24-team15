//! Task runner services.

use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use tokio::sync::oneshot::Receiver;

use crate::engine::service::runner::backend::docker;
use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::backend::Reply;
use crate::engine::Task;

pub mod backend;

/// A submitted task handle.
#[derive(Debug)]
pub struct Handle {
    /// The callback that is executed when a task is completed.
    pub callback: Receiver<Reply>,
}

/// A generic task runner.
#[derive(Debug)]
pub struct Runner {
    /// The task runner itself.
    runner: Box<dyn Backend>,

    /// The list of submitted tasks.
    pub tasks: FuturesUnordered<BoxFuture<'static, ()>>,
}

impl Runner {
    /// Creates a Docker-backed [`Runner`].
    ///
    /// # Panics
    ///
    /// If initialization of the [`bollard`](bollard) client fails.
    pub fn docker() -> Self {
        Self {
            runner: Box::new(docker::Runner::try_new().unwrap()),
            tasks: Default::default(),
        }
    }

    /// Submits a task to be executed by the backend.
    pub fn submit(&self, task: Task) -> Handle {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let a = Box::pin(self.runner.run(task, tx));
        self.tasks.push(a);

        Handle { callback: rx }
    }
}
