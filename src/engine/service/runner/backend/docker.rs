//! A docker runner service.

use crate::engine::task::Execution;
use crate::engine::Task;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::LogsOptions;
use bollard::container::LogOutput;
use bollard::container::StartContainerOptions;
use bollard::container::WaitContainerOptions;
use bollard::errors::Error;
use futures::select;
use futures::StreamExt;
use futures::FutureExt;
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

    /// Submit all tasks
    pub async fn submit(&mut self, task: Task) -> Result<Vec<i64>, Error> {
        let mut results = Vec::new();
        
        for executor in task.executions() {
            let result = self.submit_task(executor).await?;
            results.push(result);
        }
    
        Ok(results)
    }
        
    /// Submit a single task
    async fn submit_task(&self, executor: &Execution) -> Result<i64, Error> {
        let name = (1..=3)
            .map(|_| random_word::r#gen(Lang::En))
            .collect::<Vec<_>>()
            .join("-");

        let create_options = Some(CreateContainerOptions {
            name: name.clone(),
            ..Default::default()
        });

        let config = Config {
            image: Some(executor.image()),
            cmd: Some(executor.args().into_iter().map(|s| s.as_str()).collect()),
            ..Default::default()
        };

        // Create docker container
        let job = self.docker.create_container(create_options, config).await?;
        println!("{job:?}");
            
        // Start docker container
        self.docker.start_container(&name, None::<StartContainerOptions<String>>).await?;

        // Setup logs
        let stdout_path = executor.stdout().map(Path::new);
        let stderr_path = executor.stderr().map(Path::new);
            
        let log_options = LogsOptions::<String>{
            follow: true,
            stdout: executor.stdout().is_some(),
            stderr: executor.stderr().is_some(),
            ..Default::default()
        };
        
        let mut stdout_file = stdout_path.map(|path| File::create(path).expect("Failed to create stdout file"));
        let mut stderr_file = stderr_path.map(|path| File::create(path).expect("Failed to create stderr file"));
        
        let mut logs_stream = self.docker.logs(&name, Some(log_options));
        let mut wait_stream = self.docker.wait_container(&name, None::<WaitContainerOptions<String>>);

        let mut exit_code = None;
        
        // Loop through processing the stream and any final return from the container
        loop {
            select! {
                log_result = logs_stream.next().fuse() => match log_result {
                    Some(Ok(LogOutput::StdOut {message})) => { 
                        if let Some(file) = &mut stdout_file {
                            file.write_all(&message).expect("Failed to write to stdout file");
                        }
                    }
                    Some(Ok(LogOutput::StdErr {message})) => {
                        if let Some(file) = &mut stderr_file {
                            file.write_all(&message).expect("Failed to write to stdout file");
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => eprintln!("Error reading log: {:?}", e),
                    None => break // stream ended
                },
                wait_result = wait_stream.next().fuse() => match wait_result {
                    Some(Ok(wait_response)) => {
                        exit_code = Some(wait_response.status_code);
                        break;
                    }
                    Some(Err(e)) => return Err(e),
                    None => break, // This should not happen under normal circumstances
                }

            }
        }
        
        // Cleanup
        self.docker.remove_container(&name, None).await?;
        
        Ok(exit_code.unwrap_or(-1))
    }
}
