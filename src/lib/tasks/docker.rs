use super::types::{Config, DockerClient};
use crate::lib::tasks::types::DockerResult;
use bollard::{
    Docker,
    container::{CreateContainerOptions, StartContainerOptions},
    image::CreateImageOptions,
    secret::{HostConfig, Resources, RestartPolicy, RestartPolicyNameEnum},
};
use futures_util::stream::StreamExt;
use std::{error::Error, io::Write};

impl DockerResult {
    pub fn with_error(err: Box<dyn Error>) -> Self {
        DockerResult {
            container_id: None,
            action: None,
            result: None,
            error: Some(err),
        }
    }

    pub fn success(container_id: String, action: String, result: String) -> Self {
        DockerResult {
            container_id: Some(container_id),
            action: Some(action),
            result: Some(result),
            error: None,
        }
    }
}

impl DockerClient {
    pub fn new(config: Config) -> Option<Self> {
        let docker_client = Docker::connect_with_unix_defaults().ok()?;

        Some(DockerClient {
            client: docker_client,
            config,
        })
    }

    pub async fn run(&self) -> DockerResult {
        // Image pull
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
                    return DockerResult::with_error(Box::new(e));
                }
            }
        }

        println!("\nImage pulled: {}", self.config.image);

        // Restart policy
        let restart_policy = RestartPolicy {
            name: Some(
                self.config
                    .restart_policy
                    .parse()
                    .unwrap_or_else(|_| RestartPolicyNameEnum::NO),
            ),
            maximum_retry_count: None,
        };

        // Resources
        let resources: Resources = Resources {
            memory: Some(self.config.memory),
            nano_cpus: Some((self.config.cpu * 1_000_000_000.0) as i64),
            ..Default::default()
        };

        // Host Config
        let host_config: HostConfig = HostConfig {
            restart_policy: Some(restart_policy),
            nano_cpus: resources.nano_cpus,
            memory: resources.memory,
            publish_all_ports: Some(true),
            ..Default::default()
        };

        // Container Config
        let container_config = bollard::container::Config {
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
            host_config: Some(host_config),
            ..Default::default()
        };

        // Container creation options
        let options = Some(CreateContainerOptions {
            name: self.config.name.clone(),
            ..Default::default()
        });

        // Create container
        let create_result = self
            .client
            .create_container(options, container_config)
            .await;

        // Handle container creation result
        let resp_id = match create_result {
            Ok(resp) => {
                println!("Container created successfully: {}", resp.id);
                resp.id
            }
            Err(e) => {
                eprintln!("Error creating container: {:?}", e);
                return DockerResult::with_error(Box::new(e));
            }
        };

        // Start container
        println!("Starting container: {}", resp_id);
        match self
            .client
            .start_container(&resp_id, None::<StartContainerOptions<String>>)
            .await
        {
            Ok(_) => println!("Container {} started successfully.", self.config.name),
            Err(e) => {
                eprintln!("Error starting container {}: {:?}", self.config.name, e);
                return DockerResult::with_error(Box::new(e));
            }
        }

        println!("Container started successfully: {}", resp_id);

        DockerResult::success(resp_id, "Start".to_string(), "success".to_string())
    }

    pub async fn stop(&self, container_id: &str) -> DockerResult {
        println!("Stopping container: {}", container_id);
        let stop_result = self.client.stop_container(container_id, None).await;
        
        match stop_result {
            Ok(_) => {
                println!("Container stopped successfully: {}", container_id);
                DockerResult::success(
                    container_id.to_string(),
                    "Stop".to_string(),
                    "success".to_string(),
                )
            }
            Err(e) => {
                eprintln!("Error stopping container {}: {:?}", container_id, e);
                DockerResult::with_error(Box::new(e))
            }
        }
    }
}
