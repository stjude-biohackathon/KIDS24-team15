//! A docker runner service.

use std::collections::HashMap;
use crate::engine::Task;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::LogsOptions;
use bollard::container::LogOutput;
use bollard::container::StartContainerOptions;
use bollard::errors::Error;
use futures::stream::StreamExt;
use random_word::Lang;

/// The default runner service name.
pub const DEFAULT_SERVICE_NAME: &str = "docker";

/// A local execution backend.
#[derive(Debug)]
pub struct Docker {
    /// The docker handle.
    docker: bollard::Docker,
}

impl Docker {
    /// Attempts to create a new local execution backend.
    pub fn try_new() -> std::result::Result<Self, Error> {
        let docker = bollard::Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    /// Submits a task.
    pub async fn submit(&mut self, task: Task) {
        for executor in task.executions() {
            // Create stdout/stderr files if required
            let stdout_path = executor.stdout().map(|s| Path::new(s));
            let mut stdout_file = stdout_path.map(|path| File::create(path).expect("Failed to create stdout file"));
            let stderr_path = executor.stderr().map(|s| Path::new(s));
            let mut stderr_file = stderr_path.map(|path| File::create(path).expect("Failed to create stderr file"));

            let name = (1..=3)
                .map(|_| random_word::r#gen(Lang::En))
                .collect::<Vec<_>>()
                .join("-");

            let options = Some(CreateContainerOptions {
                name: name.clone(),
                ..Default::default()
            });

            let mut host_config = bollard::models::HostConfig::default();
            if let Some(ram_gb) = task.resources().unwrap().ram_gb() {
                host_config.memory = Some((ram_gb * 1024. * 1024. * 1024.) as i64);
            }

            if let Some(cpu_cores) = task.resources().unwrap().cpu_cores() {
                host_config.cpu_count = Some(cpu_cores as i64);
            }
            
            if let Some(disk_gb) = task.resources().unwrap().disk_gb() {
                let mut storage_opt: HashMap<String, String> = HashMap::new();
                storage_opt.insert("size".to_string(), disk_gb.to_string());
                host_config.storage_opt = Some(storage_opt);
            };

            let config = Config {
                image: Some(executor.image()),
                cmd: Some(executor.args().into_iter().map(|s| s.as_str()).collect()),
                host_config: Some(host_config),
                ..Default::default()
            };

            let job = self.docker.create_container(options, config).await.unwrap();

            println!("{job:?}");

            // Capture logs
            let options = LogsOptions::<String>{
                stdout: executor.stdout().is_some(),
                stderr: executor.stderr().is_some(),
                ..Default::default()
            };
            let mut logs_stream = self.docker.logs(&name, Some(options));
            
            self.docker
                .start_container(&name, None::<StartContainerOptions<String>>)
                .await
                .unwrap();
            
            while let Some(log_result) = logs_stream.next().await {
                match log_result {
                    Ok(LogOutput::StdOut {message}) => { 
                        if let Some(file) = &mut stdout_file {
                            file.write_all(&message).expect("Failed to write to stdout file");
                        }
                    }
                    Ok(LogOutput::StdErr {message}) => {
                        if let Some(file) = &mut stderr_file {
                            file.write_all(&message).expect("Failed to write to stdout file");
                        }
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("Error reading log: {:?}", e),   
                }
            }
        }
    }
}
