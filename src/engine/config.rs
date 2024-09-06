//! Global config options and loading from .crankshaft

use std::path::Path;

use config::ConfigError;
use serde::{Deserialize, Serialize};

use crate::engine::service::runner::backend;

/// The config loaded from a global file.
/// Currently contains just a list of available backends
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// All backends that exist
    pub backends: Vec<backend::Config>,
}

impl Config {
    /// Loads a new configuration file from a path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let file = config::File::<_, _>::from(path);

        let settings = config::Config::builder().add_source(file);
        settings.build()?.try_deserialize()
    }

    /// Loads a config from a test fixture.
    #[cfg(test)]
    pub fn fixture(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        use std::path::PathBuf;

        let mut result = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test/fixtures/config/",
        ));
        result.push(path.as_ref());
        Self::new(result)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn loading_file_returns_valid_backends() {
        let config = Config::fixture("full.toml").unwrap();
        assert_eq!(config.backends.len(), 3)
    }

    #[test]
    fn loading_config_holds_valid_fields() {
        let config = Config::fixture("full.toml").unwrap();
        let backend = &config.backends[1];

        assert_eq!(backend.name, "quux");
        assert_eq!(backend.default_cpu, Some(1));
        assert_eq!(backend.default_ram, Some(1));
    }
}
