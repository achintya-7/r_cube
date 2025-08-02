use super::types::{Config, DockerClient};
use crate::lib::tasks::types::{DockerError, DockerResponse, DockerResult};
use bollard::{
    Docker,
    container::{CreateContainerOptions, StartContainerOptions},
    image::CreateImageOptions,
    secret::{HostConfig, Resources, RestartPolicy, RestartPolicyNameEnum},
};
use error_stack::Report;
use futures_util::stream::StreamExt;
use std::{error::Error, io::Write};

impl DockerClient {
    pub fn new(config: Config) -> Option<Self> {
        let docker_client = Docker::connect_with_unix_defaults().ok()?;
        Some(DockerClient {
            client: docker_client,
            config,
        })
    }

    async fn pull_image(&self) -> Result<(), Box<dyn Error>> {
        println!("Pulling image: {}", self.config.image);
        let mut stream = self.client.create_image(
            Some(CreateImageOptions {
                from_image: self.config.image.clone(),
                ..Default::default()
            }),
            None,
            None,
        );

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(info) => {
                    if let Some(status) = info.status {
                        print!("\r{}", status);
                        std::io::stdout().flush().unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("\nError during image pull stream: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }

        println!("\nImage pulled: {}", self.config.image);
        Ok(())
    }

    fn host_config(&self) -> HostConfig {
        let restart_policy = RestartPolicy {
            name: Some(
                self.config
                    .restart_policy
                    .parse()
                    .unwrap_or_else(|_| RestartPolicyNameEnum::NO),
            ),
            maximum_retry_count: None,
        };

        let resources = Resources {
            memory: Some(self.config.memory),
            nano_cpus: Some((self.config.cpu * 1_000_000_000.0) as i64),
            ..Default::default()
        };

        HostConfig {
            restart_policy: Some(restart_policy),
            nano_cpus: resources.nano_cpus,
            memory: resources.memory,
            publish_all_ports: Some(true),
            ..Default::default()
        }
    }

    fn container_config(&self) -> bollard::container::Config<String> {
        bollard::container::Config {
            image: Some(self.config.image.clone()),
            env: Some(self.config.env.clone()),
            exposed_ports: Some(
                self.config
                    .exposed_ports
                    .clone()
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key.to_string(),
                            value.into_iter().map(|_| ((), ())).collect(),
                        )
                    })
                    .collect(),
            ),
            host_config: Some(self.host_config()),
            ..Default::default()
        }
    }

    pub async fn run(&self) -> DockerResult {
        if let Err(e) = self.pull_image().await {
            return Err(Report::new(DockerError::ImagePullError(e.to_string())));
        }

        let options = Some(CreateContainerOptions {
            name: self.config.name.replace(' ', "-"),
            ..Default::default()
        });

        let container_id = match self
            .client
            .create_container(options, self.container_config())
            .await
        {
            Ok(resp) => {
                println!("Container created successfully: {}", resp.id);
                resp.id
            }
            Err(e) => {
                eprintln!("Error creating container: {:?}", e);
                return Err(Report::new(DockerError::ContainerCreationError(format!(
                    "Failed to create container: {}",
                    e
                ))));
            }
        };

        println!("Starting container: {}", container_id);

        if let Err(e) = self
            .client
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
        {
            eprintln!("Error starting container {}: {:?}", self.config.name, e);
            return Err(Report::new(DockerError::ContainerStartError(format!(
                "Failed to start container: {}",
                e
            ))));
        }

        println!("Container {} started successfully.", self.config.name);
        Ok(DockerResponse {
            error: None,
            action: Some("Start".to_string()),
            container_id: Some(container_id),
        })
    }

    pub async fn stop(&self, container_id: &str) -> DockerResult {
        println!("Stopping container: {}", container_id);
        match self.client.stop_container(container_id, None).await {
            Ok(_) => {
                println!("Container stopped successfully: {}", container_id);
                Ok(DockerResponse {
                    error: None,
                    action: Some("Stop".to_string()),
                    container_id: Some(container_id.to_string()),
                })
            }
            Err(e) => {
                eprintln!("Error stopping container {}: {:?}", container_id, e);
                Err(Report::new(DockerError::ContainerStopError(format!(
                    "Failed to stop container: {}",
                    e
                ))))
            }
        }
    }
}
