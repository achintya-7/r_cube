use std::collections::HashMap;
use std::error::Error;

use bollard::Docker;
use lib::tasks::types::Config;
use lib::tasks::types::DockerClient;
use lib::tasks::types::State;
use lib::tasks::types::Task;
use lib::tasks::types::TaskEvent;

mod lib {
    pub mod manager;
    pub mod tasks;
    pub mod worker;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let docker_client = Docker::connect_with_unix_defaults().unwrap();

    let config = Config {
        image: "alpine:latest".to_string(),
        restart_policy: "no".to_string(),
        memory: 128 * 1024 * 1024, // 128 MB
        cpu: 0.5,                  // Half a CPU
        env: vec!["MY_ENV_VAR=hello".to_string()],
        exposed_ports: HashMap::from([
            ("80/tcp".to_string(), HashMap::new()),
            ("443/tcp".to_string(), HashMap::new()),
        ]),
        name: "my_rust_container_from_go".to_string(),
        ..Default::default()
    };

    let my_docker = DockerClient::new(config).expect("Failed to create Docker client");
    let result = my_docker.run().await;

    match result.error {
        Some(ref e) => eprintln!("Error starting container: {:?}", e),
        None => println!("Container started successfully!"),
    }

    println!("Docker run result: {:?}", result);

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    if let Some(container_id) = result.container_id {
        let stop_result = my_docker.stop(&container_id).await;
        println!("Docker stop result: {:?}", stop_result);
    } else {
        eprintln!("Error: No container ID found to stop.");
    }

    Ok(())
}
