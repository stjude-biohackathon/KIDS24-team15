//! A docker runner service.

use crate::engine::Task;

use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::StartContainerOptions;
use bollard::errors::Error;
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
            let name = (1..=3)
                .map(|_| random_word::r#gen(Lang::En))
                .collect::<Vec<_>>()
                .join("-");

            let options = Some(CreateContainerOptions {
                name: name.clone(),
                ..Default::default()
            });

            let config = Config {
                image: Some(executor.image()),
                cmd: Some(executor.args().into_iter().map(|s| s.as_str()).collect()),
                ..Default::default()
            };

            let job = self.docker.create_container(options, config).await.unwrap();

            println!("{job:?}");

            self.docker
                .start_container(&name, None::<StartContainerOptions<String>>)
                .await
                .unwrap();
        }
    }
}
