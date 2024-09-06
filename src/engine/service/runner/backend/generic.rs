//! Generic backend implementation

use std::{collections::HashMap, process::Command, sync::Arc};

use async_trait::async_trait;
use futures::FutureExt;
use nonempty::NonEmpty;
use regex;
use tokio::sync::oneshot::Sender;

use crate::engine::service::runner::backend;
use crate::engine::service::runner::backend::config::substitute_placeholders;
use crate::engine::service::runner::backend::config::BackendType;
use crate::engine::service::runner::backend::Backend;
use crate::engine::service::runner::backend::ExecutionResult;
use crate::engine::service::runner::backend::Reply;
use crate::engine::Task;

/// A generic backend
#[derive(Debug)]
pub struct GenericBackend {
    /// All runtime attributes
    pub runtime_attributes: Option<HashMap<String, String>>,
    /// Default cpu count
    pub default_cpu: Option<u32>,
    /// Default ram amount in mb
    pub default_ram_mb: Option<u32>,
    /// command to run on submit
    pub submit: String,
    /// job id regex
    ///
    /// This is used to extract the job id from the STDOUT of the submit command.
    /// It should have exactly one capture group.
    pub job_id_regex: Option<String>,
    /// monitor command for checking alive-ness
    pub monitor: Option<String>,
    /// frequency to monitor the job in seconds
    pub monitor_frequency: Option<u32>,
    /// kill command for killing a job
    pub kill: Option<String>,
}

impl GenericBackend {
    /// Generates a generic backend from its designated configuration
    pub fn from_config(conf: backend::Config) -> Option<Self> {
        // If BackendConfig is not of type generic, return None(Or Err)
        if let BackendType::Generic(generic_backend) = conf.kind {
            Some(Self {
                runtime_attributes: conf.runtime_attrs,
                default_cpu: conf.default_cpu,
                default_ram_mb: conf.default_ram,
                submit: generic_backend.submit,
                job_id_regex: generic_backend.job_id_regex,
                monitor: generic_backend.monitor,
                monitor_frequency: generic_backend.monitor_frequency,
                kill: generic_backend.kill,
            })
        } else {
            None
        }
    }

    /// Generates a process result from an incoming task
    pub async fn process_command(
        &self,
        substitutions: &mut HashMap<String, String>,
    ) -> Option<ExecutionResult> {
        if let Some(cpu) = self.default_cpu {
            substitutions
                .entry("cpu".to_string())
                .or_insert(cpu.to_string());
        }
        if let Some(ram) = self.default_ram_mb {
            substitutions
                .entry("memory_mb".to_string())
                .or_insert(ram.to_string());
        }

        let submit_command = substitute_placeholders(&self.submit, substitutions);
        let submit_output = Command::new("sh")
            .arg("-c")
            .arg(submit_command)
            .output()
            .expect("Failed to run command");

        let job_id_regex_str = self.job_id_regex.clone().unwrap();
        let submit_stdout = String::from_utf8(submit_output.stdout).ok().unwrap();
        let job_id = regex::Regex::new(job_id_regex_str.as_str())
            .ok()
            .unwrap()
            .captures(&submit_stdout)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        substitutions.insert("job_id".to_string(), job_id.to_string());

        let monitor_command =
            substitute_placeholders(&self.monitor.clone().unwrap(), substitutions);

        // loop while job is running.
        // monitor_command should return a non-zero exit code when the job is done
        loop {
            let monitor_output = Command::new("sh")
                .arg("-c")
                .arg(monitor_command.clone())
                .output()
                .expect("Failed to run command");

            if monitor_output.status.code()? != 0 {
                break;
            }
            // sleep for monitor_frequency seconds
            tokio::time::sleep(std::time::Duration::from_secs(
                self.monitor_frequency.unwrap_or(5).into(),
            ))
            .await;
        }

        // TODO: collect job output. In meantime, just return the status code
        // and the stdout/stderr of the submit command
        Some(ExecutionResult {
            status: submit_output.status.code()? as i64,
            stdout: submit_stdout,
            stderr: String::from_utf8(submit_output.stderr).ok()?,
        })
    }

    /// Wraps the GenericBackend in an Arc and returns the GenericRunner from it
    pub fn to_runner(self) -> GenericRunner {
        GenericRunner {
            client: Arc::new(self),
        }
    }

    /// Generates a generic backend from a config file path
    pub fn from_config_file() {
        todo!()
    }
}

/// A generic backend runner
#[derive(Debug)]
pub struct GenericRunner {
    /// An Arc to the underlying Backend
    client: Arc<GenericBackend>,
}

impl GenericRunner {
    /// Creates a new GenericRunner from a GenericBackend
    pub fn new(client: GenericBackend) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl Backend for GenericRunner {
    fn run(&self, task: Task, cb: Sender<super::Reply>) -> futures::future::BoxFuture<'static, ()> {
        let client = self.client.clone();

        async move {
            let mut results: Option<NonEmpty<ExecutionResult>> = None;
            for exec in task.executions() {
                let mut substitutions = match &client.runtime_attributes {
                    Some(attributes) => attributes.clone(),
                    None => HashMap::new(),
                };

                let command = exec
                    .args()
                    .into_iter()
                    .map(|cmd| cmd.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                substitutions.insert("script".to_string(), command);

                if let Some(cwd) = exec.workdir() {
                    substitutions.insert("cwd".to_string(), cwd.to_string());
                }

                if let Some(resources) = task.resources() {
                    if let Some(gb) = resources.ram_gb() {
                        substitutions.insert(
                            "memory_mb".to_string(),
                            ((gb * 1000f64) as usize).to_string(),
                        );
                    }
                }

                let execution_result = client.process_command(&mut substitutions).await.unwrap();

                results = match results {
                    Some(mut results) => {
                        results.push(execution_result);
                        Some(results)
                    }
                    None => Some(NonEmpty::new(execution_result)),
                }
            }

            let _ = cb.send(Reply {
                executions: Some(results.expect("at least one execution to be run")),
            });
        }
        .boxed()
    }
}
