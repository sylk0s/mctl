use bollard::{
    Docker,
    exec::CreateExecOptions,
    container::{LogsOptions, LogOutput},
    errors::Error };
use futures::Stream;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub id: String,
    pub path: String,
    pub port: u16,
}

const PATH: &str = "/home/sylkos/servers";
const COMPOSE: &str = "/home/sylkos/docker-compose.yml";

impl Server {
    pub fn new(name: String, path: Option<String>, port_arg: Option<u16>, ports: Option<Vec<u16>>) -> Server {
        let path = if let Some(p) = path {
                        p 
                    } else {
                        format!("{PATH}/{}", name)
                    };
        
        let path_obj = Path::new(&path);

        if !path_obj.exists() {
            std::fs::create_dir_all(path.clone()).expect("Error creating a new directory.");
        }

        // if compose doesn't exists, assign the port in the call, if it doesn't exist, assign the next
        // available port above 25565

        let compose_str = format!("{path}/docker-compose.yml"); 
        let compose = Path::new(&compose_str);

        if !compose.exists() {
            fs::File::create(&compose_str).expect("Error creating docker compose");
            std::fs::copy(COMPOSE, compose_str.clone()).expect("Error copying default contents of docker compose"); 
        }

        let ports = if let Some(p) = ports {
            p
        } else {
            Vec::new()
        };

        let compose_file = fs::read_to_string(compose_str).unwrap();
        let mut compose: Compose = serde_yaml::from_str(&compose_file).unwrap();

        let port_from_file = compose.services.mc.ports.get(0).unwrap().split(":").next().unwrap().parse::<u16>().unwrap();

        let port = if let Some(p) = port_arg { 
            // Check to confirm that the port doesn't already overlap with any other ports
            if ports.iter().any(|e| e == &p) {
                println!("Warning, server is already running on this port!"); 
            }
            p 
        } else { 
            // Find the next available port above 31000
            // TODO update for config
            if port_from_file == 25565 {
                ports.iter().fold(31000, |a, b| {
                    if a+1 == *b { *b } else {
                        a  
                    }
                }) + 1
            } else {
                port_from_file
            }
        };

        let output = Command::new("docker")
            .arg("compose")
            .arg("up")
            .arg("-d")
            .stdin(Stdio::piped())
            .output().unwrap();

        println!("{:?}", output);
        // parse output into ID
        let id = "".to_string();

        // add to Servers
        Server {
            name,
            path,
            id,
            port,
        }
    }
    pub async fn send_command(&self, cmd: Vec<String>) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        let full_cmd = cmd.iter().fold(vec!["rcon-cli"], |mut acc, x| { acc.push(x.as_str()); acc });
        let exec = docker
        .create_exec(
            &self.id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(full_cmd),
                ..Default::default()
            },
        )
        .await.unwrap()
        .id;

        if let Ok(_) = docker.start_exec(&exec, None).await {
            return Ok(String::from("Successfully sent cmd to container"));
        } else {
            return Err(String::from("Failed to send cmd to container"));
        }
    }

    pub async fn start(&self) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();
        
        if let Ok(_) = docker.start_container::<String>(&self.id, None).await {
            return Ok(String::from("Successfully started the container"));
        } else {
            return Err(String::from("Failed to start the container"));
        }
    }

    pub async fn stop(&self) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        if let Ok(_) = docker.stop_container(&self.id, None).await {
            return Ok(String::from("Successfully stopped the container"));
        } else {
            return Err(String::from("Failed to stop the container"));
        }
    }

    pub fn status(&self) {
        unimplemented!();
    }

   pub fn output(&self) -> impl Stream<Item = Result<LogOutput, Error>> {
       #[cfg(unix)]
       let docker = Docker::connect_with_socket_defaults().unwrap();

       let options = Some(LogsOptions::<String>{
            stdout: true,
            since: Utc::now().timestamp(),
            follow: true,
            ..Default::default()
        });

        docker.logs(&self.id, options)
    }
}

// Sketch thing for yaml

#[derive(Serialize, Deserialize, Debug)]
struct Compose {
    services: Services,  
}

#[derive(Serialize, Deserialize, Debug)]
struct Services {
    mc: Mc,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mc {
    ports: Vec<String>,
}
