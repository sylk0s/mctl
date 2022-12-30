use bollard::{
    Docker,
    exec::CreateExecOptions,
    container::{LogsOptions, LogOutput},
    errors::Error };
use futures::{Stream, stream::StreamExt, future};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::prelude::*;
use regex::Regex;
use hyper::body::Bytes;

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub id: String,
    pub path: String,
    pub port: u16,
}

const PATH: &str = "/home/sylkos/servers";
const COMPOSE: &str = "/home/sylkos/servers/docker-compose.yml";

impl Server {
    // Maybe for this arg do the nice builder thing for all the optionals
    pub fn new(name: String, path: Option<String>, port_arg: Option<u16>, ports: Option<Vec<u16>>, version: Option<String>, server_type: Option<String>) -> Server {
        let path = if let Some(p) = path {
                        p 
                    } else {
                        format!("{PATH}/{}", name)
                    };

        println!("Path: {path}");
        
        let path_obj = Path::new(&path);

        if !path_obj.exists() {
            println!("No path found, making a new path");
            std::fs::create_dir_all(path.clone()).expect("Error creating a new directory.");
        }

        // if compose doesn't exists, assign the port in the call, if it doesn't exist, assign the next
        // available port above 25565

        let compose_str = format!("{path}/docker-compose.yml"); 
        let compose = Path::new(&compose_str);

        println!("Compose Path: {compose_str}");

        if !compose.exists() {
            println!("Compose file doesn't exist at path");
            fs::File::create(&compose_str).expect("Error creating docker compose");
            fs::copy(COMPOSE, compose_str.clone()).expect("Error copying default contents of docker compose"); 
        }

        let ports = if let Some(p) = ports {
            p
        } else {
            Vec::new()
        };

        println!("Reading compose file to string");
        let compose_file = fs::read_to_string(compose_str.clone()).unwrap();
        let mut compose: Compose = serde_yaml::from_str(&compose_file).unwrap();

        let port_from_file = compose.services.mc.ports.get(0).unwrap().split(":").next().unwrap().parse::<u16>().unwrap();
        println!("Port from file is: {port_from_file}");
        let port = if let Some(p) = port_arg { 
            p 
        } else { 
            // Find the next available port above 31000
            if port_from_file == 25565 {
                ports.iter().fold(31000-1, |a, b| {
                    if a+1 == *b { *b } else {
                        a  
                    }
                }) + 1
            } else {
                port_from_file
            }
        };

        println!("Port: {port}");

        compose.services.mc.ports = vec![format!("{port}:25565")];

        if let Some(v) = version {
            compose.services.mc.environment.VERSION = v;
        }

        if let Some(t) = server_type {
            compose.services.mc.environment.TYPE = Some(t);
        }
        
        println!("Writing updated compose to compose file");
        println!("Compose path: {compose_str}");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .append(false)
            .open(compose_str)
            .unwrap();
        file.write(serde_yaml::to_string(&compose).expect("tostr").as_bytes()).expect("error writing");

        if ports.iter().any(|e| e == &port) {
            println!("Warning, server is already registered on this port!"); 
        }

        let output = Command::new("docker")
            .arg("compose")
            .arg("up")
            .arg("-d")
            .current_dir(&path)
            .output().unwrap().stderr;


        // btw this doesnt work if the container is already running, add a handler for that?
        let str_out = std::str::from_utf8(&output).unwrap();
        println!("Output from docker compose: \n{str_out}");
        let id = str_out.split("\n").skip_while(|e| !e.starts_with("Container")).next().expect("aaa").split(" ").skip(1).next().expect("bbb").to_string();
        println!("Id: {:?}", id);

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

    pub fn clean_output(&self) -> impl Stream<Item = Result<hyper::body::Bytes, bollard::errors::Error>> {
        self.output().filter_map(|msg| {
            future::ready(match msg {
                Ok(m) => {
                    let re1 = Regex::new(r"<.*>.*").unwrap();
                    let re2 = Regex::new(r"left the game").unwrap();
                    let re3 = Regex::new(r"joined the game").unwrap();
                    if re1.is_match(&m.to_string()) || re2.is_match(&m.to_string()) || re3.is_match(&m.to_string()) {
                        let text = m.to_string().split(" ").skip(3).fold(String::new(), |a, b| format!("{a} {b}"));
                        Some(Ok(Bytes::from(text)))
                    } else {
                        None
                    }
                        }
                Err(_) => None,
            })
        })
    }
}

// Sketch thing for yaml
#[derive(Serialize, Deserialize, Debug)]
struct Compose {
    version: String,
    services: Services,  
}

#[derive(Serialize, Deserialize, Debug)]
struct Services {
    mc: Mc,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mc {
    image: String,
    ports: Vec<String>,
    environment: Env,
    tty: bool,
    stdin_open: bool,
    restart: String,
    volumes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct Env {
    EULA: String,
    VERSION: String,
    TYPE: Option<String>,
    MOTD: Option<String>,
    DIFFICULTY: Option<String>,
    ENABLE_WHITELIST: Option<String>,
    WHITELIST: Option<String>,
    OPS: Option<String>,
    MAX_PLAYERS: Option<u16>,
    SEED: Option<String>,
    MODE: Option<String>,
    CUSTOM_SERVER: Option<String>,
}
