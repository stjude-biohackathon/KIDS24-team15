//! Configuration for different types of backends

use std::{
    collections::HashMap,
    process::{Command, Output, Stdio},
};

use serde::{Deserialize, Serialize};

/// Configuration for an arbitrary backend
#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl BackendConfig {
    /// Submits a backend based on its config. Likely this method will be removed and the branch for Generic will be moved to GenericBackend's submit method.
    /// Instead of this method we should have a to_backend() method or something similar that creates a Box<dyn Backend> based on config.
    pub fn submit(
        &self,
        replacements: &mut HashMap<String, String>,
        left_placeholder: &str,
        right_placeholder: &str,
    ) -> Option<Output> {
        // Replace default flags only if it isn't already set

        if let Some(cpu) = self.default_cpu {
            if !replacements.contains_key("cpu") {
                replacements.insert("cpu".to_string(), cpu.to_string());
            }
        }

        if let Some(ram) = self.default_ram {
            if !replacements.contains_key("ram") {
                replacements.insert("cpu".to_string(), ram.to_string());
            }
        }

        match &self.kind {
            BackendType::Generic(generic) => {
                let mut command_str = generic.command.clone();
                for (key, value) in replacements {
                    let placeholder_key =
                        format!("{}{}{}", left_placeholder, key, right_placeholder);
                    command_str = command_str.replace(&placeholder_key, value);
                }

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command_str)
                    // We could set stdout and stderr to be the same file system we did with Docker
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("Failed to run command");
                Some(output)
            }
            BackendType::Docker(_docker) => {
                // Because this method is really only to test the generic backend's submitting, I'm going to leave Docker unimplemented unless otherwise requested
                None
            }
        }
    }
}

/// An enum representing extra metadata supplied in the config file depending on the kind of backend
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum BackendType {
    /// Generic backend config, will contain the shell script string for submitting
    Generic(GenericBackendConfig),
    /// Docker config details
    Docker(DockerBackendConfig),
}

/// Extra attributes for Generic Backends
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericBackendConfig {
    command: String,
}

/// Extra attributes for Docker backends
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DockerBackendConfig;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::engine::config::Config;

    #[test]
    fn simple_generic_config_loads_and_runs() {
        let config = Config::load_from_file("configs/generic_simple.toml")
            .expect("Load from example config");
        let backend = &config.backends[0];
        let mut replacements = HashMap::new();
        replacements.insert("name".to_string(), "Kids24".to_string());

        let output = backend.submit(&mut replacements, "${", "}");
        assert!(output.is_some());
    }
}
