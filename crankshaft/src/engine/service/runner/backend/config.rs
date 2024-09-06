//! Configuration for different types of backends

use std::collections::HashMap;

#[cfg(test)]
use std::process::{Command, Output};

use serde::Deserialize;
use serde::Serialize;

/// The left placeholder for the backend config
const LEFT_PLACEHOLDER: &str = "~{";
/// The right placeholder for the backend config
const RIGHT_PLACEHOLDER: &str = "}";

/// Substitutes placeholders in a string with values from a hashmap
pub(crate) fn substitute_placeholders(s: &str, substitutions: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in substitutions {
        let placeholder_key = format!("{}{}{}", LEFT_PLACEHOLDER, key, RIGHT_PLACEHOLDER);
        result = result.replace(&placeholder_key, value);
    }
    result
}

/// Configuration for an arbitrary backend
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
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

impl Config {
    /// Submits a backend based on its config. Likely this method will be removed and the branch for Generic will be moved to GenericBackend's submit method.
    /// Instead of this method we should have a to_backend() method or something similar that creates a Box<dyn Backend> based on config.
    #[cfg(test)]
    pub fn submit(&self, substitutions: &mut HashMap<String, String>) -> Output {
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
                let command = substitute_placeholders(&generic.submit, substitutions);

                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .expect("failed to run command")
            }
            _ => {
                // SAFETY: this method is only to test the generic backend, so
                // any other variant should throw unreachable.
                unreachable!()
            }
        }
    }
}

/// An enum representing extra metadata supplied in the config file depending on the kind of backend
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum BackendType {
    /// Generic backend config, will contain the shell script string for submitting
    Generic(GenericBackendConfig),
    /// Docker config details
    Docker(DockerBackendConfig),
}

/// Extra attributes for Generic Backends
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenericBackendConfig {
    /// The script command that will be run on submit
    pub submit: String,
    /// The regex that will be used to extract the job id from STDOUT
    pub job_id_regex: Option<String>,
    /// The script command that will run on monitor check
    pub monitor: Option<String>,
    /// The frequency that the monitor command will run in seconds
    pub monitor_frequency: Option<u32>,
    /// The script command that will run on kill
    pub kill: Option<String>,
}

/// Extra attributes for Docker backends
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DockerBackendConfig;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::engine::config::Config;

    #[test]
    fn simple_generic_config_loads_and_runs() {
        let config = Config::fixture("generic.toml").unwrap();
        let backend = &config.backends[0];
        let mut substitutions = HashMap::new();
        substitutions.insert("name".to_string(), "Kids24".to_string());

        let output = backend.submit(&mut substitutions);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(&stdout, "Hello Kids24\n");
    }

    #[test]
    fn generic_config_with_defaults_uses_them() {
        let config = Config::fixture("generic.toml").unwrap();
        let backend = &config.backends[1];
        let mut substitutions = HashMap::new();

        let output = backend.submit(&mut substitutions);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(&stdout, "I have 4096 mb of ram\n");
    }

    #[test]
    fn generic_config_with_defaults_and_parameters_set_uses_parameters() {
        let config = Config::fixture("generic.toml").unwrap();
        let backend = &config.backends[1];
        let mut substitutions = HashMap::new();
        substitutions.insert("ram".to_string(), 2.to_string());

        let output = backend.submit(&mut substitutions);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(&stdout, "I have 2 mb of ram\n");
    }

    #[test]
    fn lsf_example() {
        let config = Config::fixture("lsf.toml").unwrap();
        let backend = &config.backends[0];
        let mut substitutions = HashMap::new();
        substitutions.extend(backend.runtime_attrs.clone().unwrap());

        match &backend.kind {
            super::BackendType::Generic(generic) => {
                let command_str = generic.submit.clone();
                let subbed = super::substitute_placeholders(&command_str, &substitutions);
                assert_eq!(subbed, "    bsub -q compbio -n 1 -cwd ~{cwd} -o ~{cwd}/stdout.lsf -e ~{cwd}/stderr.lsf -R \"rusage[mem=~{memory_mb}] span[hosts=1]\" ~{script}\n");
            }
            _ => panic!("expected generic backend"),
        }
    }
}
