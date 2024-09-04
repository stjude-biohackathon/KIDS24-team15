//! Builders for a [`Catalog`].

use indexmap::IndexMap;

use crate::engine::service::catalog::Daemon;
use crate::engine::service::catalog::Name;
use crate::engine::service::Catalog;
use crate::engine::service::Logger;
use crate::engine::service::Runner;
use crate::engine::service::Service;

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A service already exists with this name.
    ServiceExists(Name),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ServiceExists(name) => write!(f, "service already exists: {name}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The inner representation of the unbuilt catalog.
type Inner = IndexMap<Name, Service>;

/// A builder for a [`Catalog`].
#[derive(Debug, Default)]
pub struct Builder(Inner);

impl Builder {
    /// Adds a [`Logger`] to the [`Builder`].
    ///
    /// If a service with the same name already exists, a
    /// [`Error::ServiceExists`] is returned.
    pub fn add_logger(mut self, name: impl Into<String>, logger: Logger) -> Result<Self> {
        let name = Name::Logging(name.into());

        if self.0.contains_key(&name) {
            return Err(Error::ServiceExists(name));
        }

        self.0.insert(name, Service::Logger(logger));
        Ok(self)
    }

    /// Adds a [`Runner`] to the [`Builder`].
    ///
    /// If a service with the same name already exists, a
    /// [`Error::ServiceExists`] is returned.
    pub fn add_runner(mut self, name: impl Into<String>, runner: Runner) -> Result<Self> {
        let name = Name::Runner(name.into());

        if self.0.contains_key(&name) {
            return Err(Error::ServiceExists(name));
        }

        self.0.insert(name, Service::Runner(runner));
        Ok(self)
    }

    /// Builds an immutable [`Catalog`].
    pub fn build(self) -> Catalog {
        Catalog::from(
            self.0
                .into_iter()
                .map(|(name, svc)| (name, Daemon::spawn(svc)))
                .collect::<IndexMap<_, _>>(),
        )
    }
}
