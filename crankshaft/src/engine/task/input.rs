//! Task inputs.

mod builder;

use std::path::PathBuf;

pub use builder::Builder;

use tokio::{fs::File, io::AsyncReadExt};
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

impl From<PathBuf> for Contents {
    fn from(value: PathBuf) -> Self {
        let url = Url::from_file_path(value).unwrap_or_else(|_| panic!("Invalid path"));
        Contents::URL(url)
    }
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
    /// Gets a new builder for an [`Input`].
    pub fn builder() -> Builder {
        Builder::default()
    }

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

    /// Fetch file contents
    pub async fn fetch(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match &self.contents {
            Contents::Literal(content) => Ok(content.as_bytes().to_vec()),
            Contents::URL(url) => match url.scheme() {
                "file" => {
                    let path = url.to_file_path().map_err(|_| "Invalid file path")?;
                    let mut file = File::open(path).await?;
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents).await?;
                    Ok(contents)
                }
                "http" | "https" => todo!("HTTP(S) URL support not implemented"),
                "s3" => todo!("S3 URL support not implemented"),
                _ => Err("Unsupported URL scheme".into()),
            },
        }
    }
}
