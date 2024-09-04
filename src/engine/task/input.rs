//! Task inputs.

mod builder;

pub use builder::Builder;

use url::Url;

/// A type of input.
#[derive(Clone, Debug)]
pub enum Type {
    /// A file.
    File,

    /// A directory.
    Directory,
}

/// The source of an input.
#[derive(Clone, Debug)]
pub enum Contents {
    /// Contents sourced from a URL.
    URL(Url),

    /// Contents provided as a string literal.
    Literal(String),
}

/// An input to a task.
#[derive(Clone, Debug)]
pub struct Input {
    /// A name.
    name: Option<String>,

    /// A description.
    description: Option<String>,

    /// The contents.
    contents: Contents,

    /// The path to map the input to within the container.
    path: String,

    /// The type of the input.
    r#type: Type,
}

impl Input {
    /// The name of the input (if it exists).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The description of the input (if it exists).
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The contents of the input.
    pub fn contents(&self) -> &Contents {
        &self.contents
    }

    /// The path where the input should be placed within the container.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The type of the container.
    pub fn r#type(&self) -> &Type {
        &self.r#type
    }
}
