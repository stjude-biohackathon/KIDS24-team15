//! Configuration for different types of backends

use serde::{Deserialize, Serialize};

/// Configuration for an arbitrary backend
#[derive(Serialize, Deserialize, Debug)]
pub struct BackendConfig {
    /// The backend's name
    pub name: String,
    /// The backend's type
    #[serde(flatten)]
    pub kind: BackendType,
    /// The default cpu count if present
    #[serde(rename = "default-cpu", default)]
    pub default_cpu: Option<u32>,
    /// The default ram if present
    #[serde(rename = "default-ram", default)]
    pub default_ram: Option<u32>,
}

impl BackendConfig {}

/// An enum representing extra metadata supplied in the config file depending on the kind of backend
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum BackendType {
    /// Generic backend config, will contain the shell script string for submitting
    Generic(GenericBackendConfig),
    /// Docker config details
    Docker(DockerBackendConfig),
}

/// Extra attributes for Generic Backends
#[derive(Serialize, Deserialize, Debug)]
pub struct GenericBackendConfig {
    command: String,
}

/// Extra attributes for Docker backends
#[derive(Serialize, Deserialize, Debug)]
pub struct DockerBackendConfig;
