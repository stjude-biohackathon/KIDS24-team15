//! A builder for a [`Task`].

use nonempty::NonEmpty;

use crate::engine::task::execution::Execution;
use crate::engine::task::resources::Resources;
use crate::engine::task::Input;
use crate::engine::task::Output;
use crate::engine::Task;

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A required value was missing for a builder field.
    Missing(&'static str),

    /// Multiple values were provided for a singular builder field.
    Multiple(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Missing(field) => {
                write!(f, "missing required value for '{field}' in task builder")
            }
            Error::Multiple(field) => {
                write!(f, "multiple value provided for '{field}' in task builder")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// A builder for an [`Task`].
#[derive(Debug, Default)]
pub struct Builder {
    /// An optional name.
    name: Option<String>,

    /// An optional description.
    description: Option<String>,

    /// An optional list of [`Input`]s.
    inputs: Option<NonEmpty<Input>>,

    /// An optional list of [`Output`]s.
    outputs: Option<NonEmpty<Output>>,

    /// An optional set of [`Resources`].
    resources: Option<Resources>,

    /// The list of [`Executor`]s.
    executors: Option<NonEmpty<Execution>>,

    /// The list of volumes shared among executions
    volumes: Option<NonEmpty<String>>,
}

impl Builder {
    /// Attempts to add a name to the [`Builder`].
    pub fn name<S: Into<String>>(mut self, name: S) -> Result<Self> {
        let name = name.into();

        if self.name.is_some() {
            return Err(Error::Multiple("name"));
        }

        self.name = Some(name);
        Ok(self)
    }

    /// Attempts to add a description to the [`Builder`].
    pub fn description<S: Into<String>>(mut self, description: S) -> Result<Self> {
        let description = description.into();

        if self.description.is_some() {
            return Err(Error::Multiple("description"));
        }

        self.description = Some(description);
        Ok(self)
    }

    /// Attempts to extend inputs within the [`Builder`].
    pub fn extend_inputs<Iter>(mut self, inputs: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = Input>,
    {
        let mut new = inputs.into_iter();

        self.inputs = match self.inputs {
            Some(mut inputs) => {
                inputs.extend(new);
                Some(inputs)
            }
            None => {
                if let Some(input) = new.next() {
                    let mut inputs = NonEmpty::new(input);
                    inputs.extend(new);
                    Some(inputs)
                } else {
                    None
                }
            }
        };

        Ok(self)
    }

    /// Attempts to extend outputs within the [`Builder`].
    pub fn extend_outputs<Iter>(mut self, outputs: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = Output>,
    {
        let mut new = outputs.into_iter();

        self.outputs = match self.outputs {
            Some(mut outputs) => {
                outputs.extend(new);
                Some(outputs)
            }
            None => {
                if let Some(output) = new.next() {
                    let mut outputs = NonEmpty::new(output);
                    outputs.extend(new);
                    Some(outputs)
                } else {
                    None
                }
            }
        };

        Ok(self)
    }

    /// Attempts to add a resources to the [`Builder`].
    pub fn resources(mut self, resources: Resources) -> Result<Self> {
        if self.resources.is_some() {
            return Err(Error::Multiple("resources"));
        }

        self.resources = Some(resources);
        Ok(self)
    }

    /// Attempts to extend executors within the [`Builder`].
    pub fn extend_executors<Iter>(mut self, executors: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = Execution>,
    {
        let mut new = executors.into_iter();

        self.executors = match self.executors {
            Some(mut executors) => {
                executors.extend(new);
                Some(executors)
            }
            None => {
                if let Some(executor) = new.next() {
                    let mut executors = NonEmpty::new(executor);
                    executors.extend(new);
                    Some(executors)
                } else {
                    None
                }
            }
        };

        Ok(self)
    }

    /// Attempts to extend volumes within the [`Builder`].
    pub fn extend_volumes<Iter>(mut self, volumes: Iter) -> Result<Self>
    where
        Iter: IntoIterator<Item = String>,
    {
        let mut new = volumes.into_iter();

        self.volumes = match self.volumes {
            Some(mut volumes) => {
                volumes.extend(new);
                Some(volumes)
            }
            None => {
                if let Some(volume) = new.next() {
                    let mut volumes: NonEmpty<_> = NonEmpty::new(volume);
                    volumes.extend(new);
                    Some(volumes)
                } else {
                    None
                }
            }
        };

        Ok(self)
    }

    /// Consumes `self` and attempts to return a built [`Task`].
    pub fn try_build(self) -> Result<Task> {
        let executors = self
            .executors
            .map(Ok)
            .unwrap_or(Err(Error::Missing("executors")))?;

        Ok(Task {
            name: self.name,
            description: self.description,
            inputs: self.inputs,
            outputs: self.outputs,
            resources: self.resources,
            executions: executors,
            volumes: self.volumes,
        })
    }
}
