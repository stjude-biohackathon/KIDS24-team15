//! Daemons within the service catalog.

use kameo::actor::ActorRef;

use crate::engine::service::Logger;
use crate::engine::service::Runner;
use crate::engine::service::Service;

/// An indefinitely running service.
#[derive(Debug)]
pub enum Daemon {
    /// An executing logger service.
    Logger(ActorRef<Logger>),

    /// An executing task runner.
    Runner(ActorRef<Runner>),
}

impl Daemon {
    /// Spawns a new daemon from a [`Service`] definition.
    pub fn spawn(service: Service) -> Self {
        match service {
            Service::Logger(svc) => Self::Logger(kameo::spawn(svc)),
            Service::Runner(svc) => Self::Runner(kameo::spawn(svc)),
        }
    }

    /// Attempts to get a reference to the inner [`ActorRef<Logger>`].
    ///
    /// * If `self` is a [`Self::Logger`], then a reference to the inner [`ActorRef<Logger>`] wrapped in [`Some`] is returned.
    /// * Else, [`None`] is returned.
    pub fn as_logger(&self) -> Option<&ActorRef<Logger>> {
        match self {
            Self::Logger(logger) => Some(logger),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the inner [`ActorRef<Logger>`].
    ///
    /// * If `self` is a [`Self::Logger`], then the inner [`ActorRef<Logger>`] wrapped in [`Some`] is returned.
    /// * Else, [`None`] is returned.
    pub fn into_logger(self) -> Option<ActorRef<Logger>> {
        match self {
            Self::Logger(logger) => Some(logger),
            _ => None,
        }
    }

    /// Consumes `self` and returns the inner [`ActorRef<Logger>`].
    ///
    /// # Panics
    ///
    /// If `self` is not a [`Self::Logger`].
    pub fn unwrap_logger(self) -> ActorRef<Logger> {
        self.into_logger()
            .expect("expected `Logger` but got a different variant")
    }

    /// Attempts to get a reference to the inner [`ActorRef<Runner>`].
    ///
    /// * If `self` is a [`Self::Runner`], then a reference to the inner [`ActorRef<Runner>`] wrapped in [`Some`] is returned.
    /// * Else, [`None`] is returned.
    pub fn as_runner(&self) -> Option<&ActorRef<Runner>> {
        match self {
            Self::Runner(runner) => Some(runner),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the inner [`ActorRef<Runner>`].
    ///
    /// * If `self` is a [`Self::Runner`], then the inner [`ActorRef<Runner>`] wrapped in [`Some`] is returned.
    /// * Else, [`None`] is returned.
    pub fn into_runner(self) -> Option<ActorRef<Runner>> {
        match self {
            Self::Runner(runner) => Some(runner),
            _ => None,
        }
    }

    /// Consumes `self` and returns the inner [`ActorRef<Runner>`].
    ///
    /// # Panics
    ///
    /// If `self` is not a [`Self::Runner`].
    pub fn unwrap_runner(self) -> ActorRef<Runner> {
        self.into_runner()
            .expect("expected `Runner` but got a different variant")
    }
}
