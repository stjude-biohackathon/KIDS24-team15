//! Task runner services.

use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use tokio::sync::oneshot::Receiver;

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
    backend: Box<dyn Backend>,

    /// The list of submitted tasks.
    pub tasks: FuturesUnordered<BoxFuture<'static, ()>>,
}

impl Runner {
    /// Creates a new [`Runner`].
    pub fn new(backend: impl Backend) -> Self {
        Self {
            backend: Box::new(backend),
            tasks: Default::default(),
        }
    }

    /// Submits a task to be executed by the backend.
    pub fn submit(&self, task: Task) -> Handle {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tasks.push(Box::pin(self.backend.run(task, tx)));

        Handle { callback: rx }
    }
}
