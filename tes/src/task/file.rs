//! Files referenced within TES tasks.

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Default)]
pub enum Type {
    /// A file.
    #[serde(rename = "FILE")]
    #[default]
    File,

    /// A directory.
    #[serde(rename = "DIRECTORY")]
    Directory,
}
