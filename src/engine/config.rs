//! Global config options and loading from .crankshaft

use std::path::PathBuf;

use config::ConfigError;
use serde::{Deserialize, Serialize};

use super::service::runner::backend::backend_config::BackendConfig;

/// The config loaded from a global file.
/// Currently contains just a list of available backends
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// All backends that exist
    pub backends: Vec<BackendConfig>,
}

impl Config {
    /// Loads a config from the global .crankshaft file found in the user's home directory
    pub fn load_from_global_config() -> Result<Self, ConfigError> {
        let home_dir = dirs::home_dir().unwrap();
        let config_path = home_dir.join(".crankshaft");

        Self::load_from_file(config_path)
    }

    /// Loads a config file from a given path
    pub fn load_from_file<P>(path: P) -> Result<Self, ConfigError>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let config_file: config::File<_, _> = path.into();

        let settings = config::Config::builder().add_source(config_file);
        settings.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn loading_file_returns_valid_backends() {
        let config =
            Config::load_from_file("configs/example.toml").expect("Load from example config");

        assert_eq!(config.backends.len(), 3)
    }

    #[test]
    fn loading_config_holds_valid_fields() {
        let config =
            Config::load_from_file("configs/example.toml").expect("Load from example config");
        let backend = &config.backends[1];

        assert_eq!(backend.name, "quux");
        assert_eq!(backend.default_cpu, Some(1));
        assert_eq!(backend.default_ram, Some(1));
    }
}
