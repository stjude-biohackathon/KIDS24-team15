//! Configuration for different types of backends

use std::{
    collections::HashMap,
    process::{Command, Output},
};

use serde::{Deserialize, Serialize};

/// The left placeholder for the backend config
const LEFT_PLACEHOLDER: &str = "~{";
/// The right placeholder for the backend config
const RIGHT_PLACEHOLDER: &str = "}";

/// Substitutes placeholders in a string with values from a hashmap
fn substitute_placeholders(s: &str, substitutions: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in substitutions {
        let placeholder_key = format!("{}{}{}", LEFT_PLACEHOLDER, key, RIGHT_PLACEHOLDER);
        result = result.replace(&placeholder_key, value);
    }
    result
}

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
    /// The runtime attributes for the backend
    pub runtime_attrs: Option<HashMap<String, String>>,
}

impl BackendConfig {
    /// Submits a backend based on its config. Likely this method will be removed and the branch for Generic will be moved to GenericBackend's submit method.
    /// Instead of this method we should have a to_backend() method or something similar that creates a Box<dyn Backend> based on config.
    pub fn submit(&self, substitutions: &mut HashMap<String, String>) -> Option<Output> {
        // Replace default flags only if it isn't already set

        if let Some(cpu) = self.default_cpu {
            substitutions
                .entry("cpu".to_string())
                .or_insert(cpu.to_string());
        }

        if let Some(ram) = self.default_ram {
            substitutions
                .entry("ram".to_string())
                .or_insert(ram.to_string());
        }

        match &self.kind {
            BackendType::Generic(generic) => {
                let command_str = substitute_placeholders(&generic.command, substitutions);

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command_str)
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
    /// The script command that will be run on submit
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
        let mut substitutions = HashMap::new();
        substitutions.insert("name".to_string(), "Kids24".to_string());

        let output = backend
            .submit(&mut substitutions)
            .expect("Get output from generic backend");
        assert_eq!(output.stdout, b"Hello Kids24\n");
    }

    #[test]
    fn generic_config_with_defaults_uses_them() {
        let config = Config::load_from_file("configs/generic_simple.toml")
            .expect("Load from example config");
        let backend = &config.backends[1];
        let mut substitutions = HashMap::new();

        let output = backend
            .submit(&mut substitutions)
            .expect("Get output from generic backend");
        assert_eq!(output.stdout, b"I have 4096 mb of ram\n");
    }

    #[test]
    fn generic_config_with_defaults_and_parameters_set_uses_parameters() {
        let config = Config::load_from_file("configs/generic_simple.toml")
            .expect("Load from example config");
        let backend = &config.backends[1];
        let mut substitutions = HashMap::new();
        substitutions.insert("ram".to_string(), 2.to_string());

        let output = backend
            .submit(&mut substitutions)
            .expect("Get output from generic backend");
        assert_eq!(output.stdout, b"I have 2 mb of ram\n");
    }

    #[test]
    fn lsf_example() {
        let config = Config::load_from_file("configs/lsf.toml").expect("Load from example config");
        let backend = &config.backends[0];
        let mut substitutions = HashMap::new();
        substitutions.extend(backend.runtime_attrs.clone().unwrap());

        match &backend.kind {
            super::BackendType::Generic(generic) => {
                let command_str = generic.command.clone();
                let subbed = super::substitute_placeholders(&command_str, &substitutions);
                assert_eq!(subbed, "    bsub -q compbio -n 1 -g crankshaft -R \"rusage[mem=~{memory}] span[hosts=1]\" -cwd ~{cwd} -o ~{cwd}/execution/stdout.lsf -e ~{cwd}/execution/stderr.lsf /usr/bin/env bash ~{script}\n");
            }
            _ => panic!("Expected generic backend"),
        }
    }
}
