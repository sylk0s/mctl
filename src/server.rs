use bollard::{
    Docker,
    exec::CreateExecOptions,
    container::{LogsOptions, LogOutput}
};
use futures::{Stream, stream::StreamExt, future};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::prelude::*;
use regex::Regex;
use hyper::body::Bytes;
use cloudsync::{CloudSync, Unique, CLConfig};
use crate::{Config, CONF_PATH};
use craftping::{
    tokio::ping,
    Response};
use tokio::net::TcpStream;
use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub name: String,
    pub id: String,
    pub path: String,
    pub port: u16,
}

impl Server {
    // Maybe for this arg do the nice builder thing for all the optionals
    pub async fn new(
        name: String, 
        
        path: Option<String>, 
        port_arg: Option<u16>,
        ports: Option<Vec<u16>>, 
        version: Option<String>, 
        server_type: Option<String>, 
        
        //server_config: &ServerConfig,
        config: Config
        ) -> Result<Server, Error> {

        let path = if let Some(p) = path {
                        p 
                    } else {
                        format!("{}/{}", config.path, name)
                    };

        println!("Path: {path}");
        
        let path_obj = Path::new(&path);

        if !path_obj.exists() {
            println!("No path found, making a new path");
            if let Err(_) = std::fs::create_dir_all(path.clone()) {
                return Err(Error::from("Error creating a new directory"))
            };

        }

        // if compose doesn't exists, assign the port in the call, if it doesn't exist, assign the next
        // available port above 25565

        let compose_str = format!("{path}/docker-compose.yml"); 
        let compose = Path::new(&compose_str);

        println!("Compose Path: {compose_str}");

        if !compose.exists() {
            println!("Compose file doesn't exist at path");
            if let Err(_) = fs::File::create(&compose_str) {
                return Err(Error::from("Error creating docker new docker compose file"))
            };
            if let Err(_) = fs::copy(format!("{CONF_PATH}/docker-compose.yml"), compose_str.clone()) {
                return Err(Error::from("Error copying default contents of docker compose file"))
            };
        }

        let ports = if let Some(p) = ports {
            p
        } else {
            Vec::new()
        };

        println!("Reading compose file to string");

        let compose_file = if let Ok(c) = fs::read_to_string(compose_str.clone()) { c } else {
            return Err(Error::from("Error reading compose file to a string"))
        };

        let mut compose: Compose = if let Ok(c) = serde_yaml::from_str(&compose_file) { c } else {
            return Err(Error::from("Error parsing YAML from compose file"))
        };

        let def_file = if let Ok(d) = fs::read_to_string(format!("{CONF_PATH}/docker-compose.yml")) { d } else {
            return Err(Error::from("Error reading default compose file to string"))
        };

        let def: Compose = if let Ok(d) = serde_yaml::from_str(&def_file) { d } else {
            return Err(Error::from("Error parsing YAML from default compose file"))
        };

        let port_from_file = if let Some(a) = compose.services.mc.ports.get(0) {
            if let Some(b) = a.split(":").next() {
                if let Ok(c) = b.parse::<u16>() { c } else {
                    return Err(Error::from("Failed parsing port"))
                }
            } else {
                return Err(Error::from("Port string in YAML is misformatted"))
            }
        } else {
            return Err(Error::from("Could not get \"ports\" field from YAML"))       
        };

        let def_port = if let Some(a) = def.services.mc.ports.get(0) {
            if let Some(b) = a.split(":").next() {
                if let Ok(c) = b.parse::<u16>() { c } else {
                    return Err(Error::from("Failed parsing port"))
                }
            } else {
                return Err(Error::from("Port string in YAML is misformatted"))
            }
        } else {
            return Err(Error::from("Could not get \"ports\" field from YAML"))       
        };

        println!("Port from file is: {port_from_file}");
        println!("Default port is: {def_port}");

        let port = if let Some(p) = port_arg { 
            p 
        } else { 
            // Find the next available port above 31000
            // fairly certain there is a bug here
            println!("{:?}", ports);
            if port_from_file == def_port {
                println!("Finding next empty port...");
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
        let mut file = if let Ok(f) = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .append(false)
            .open(compose_str) { f } else {
                return Err(Error::from("Failed to open compose file for updating"))
            };

        // I think this one is fine to leave as an unwrap, shouldn't fail
        let yaml = if let Ok(y) = serde_yaml::to_string(&compose) { y } else {
            return Err(Error::from("Error reading {} to file"));
        };
        
        if let Err(_) = file.write(yaml.as_bytes()) {
            return Err(Error::from("Failed to write YAML object to file"));
        };

        if ports.iter().any(|e| e == &port) {
            println!("Warning, server is already registered on this port!"); 
        }

        // Add some error handling for is the server is already running on the port

        let output = if let Ok(o) = Command::new("docker")
            .arg("compose")
            .arg("up")
            .arg("-d")
            .current_dir(&path)
            .output() { o.stderr } else {
                return Err(Error::from("Failed to run docker-compose command"));
            };


        // btw this doesnt work if the container is already running, add a handler for that?
        let str_out = if let Ok(s) = std::str::from_utf8(&output) { s } else {
            return Err(Error::from("Failed to parse cmd output"))
        };
        println!("Output from docker compose: \n{str_out}");

        let id = if let Some(a) = str_out.split("\n").skip_while(|e| !e.starts_with("Container")).next() {
            if let Some(b) = a.split(" ").skip(1).next() {
                b.to_string()
            } else {
                return Err(Error::from("Couldn't find container ID"));
            }
        } else {
            return Err(Error::from("Couldn't find container ID"));
        };
        println!("Id: {:?}", id);

        // add to Servers
        let server = Server {
            name,
            path,
            id,
            port,
        };

        if let Err(_) = server.save().await {
            return Err(Error::from("Failed to save new server object to firebase"));
        };
        Ok(server)
    }

    pub async fn send_command(&self, cmd: Vec<String>) -> Result<(), Error> {
        #[cfg(unix)]
        let docker = if let Ok(d) = Docker::connect_with_socket_defaults() { d } else {
            return Err(Error::from("Couldn't connect to docker on default socket"));
        };

        let full_cmd = cmd.iter().fold(vec!["rcon-cli"], |mut acc, x| { acc.push(x.as_str()); acc });
        let exec = if let Ok(e) = docker
        .create_exec(
            &self.id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(full_cmd),
                ..Default::default()
            },
        )
        .await { e.id } else {
            return Err(Error::from("Failed creating exec for docker"))
        };

        if let Err(_) = docker.start_exec(&exec, None).await {
            return Err(Error::from("Failed to send cmd to container"));
        }
        Ok(())
    }

    pub async fn start(&self) -> Result<(), Error> {
        #[cfg(unix)]
        let docker = if let Ok(d) = Docker::connect_with_socket_defaults() { d } else {
            return Err(Error::from("Couldn't connect to docker on default socket"));
        };
        
        if let Err(_) = docker.start_container::<String>(&self.id, None).await {
            return Err(Error::from("Failed to start the container"));
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Error> {
        #[cfg(unix)]
        let docker = if let Ok(d) = Docker::connect_with_socket_defaults() { d } else {
            return Err(Error::from("Couldn't connect to docker on default socket"));
        };

        if let Err(_) = docker.stop_container(&self.id, None).await {
            return Err(Error::from("Failed to stop the container"));
        }
        Ok(())
    }

    pub async fn status(&self) -> Result<Response, Error> {
        println!("Attempting to get status");
        let hostname = "localhost";
        let port = self.port;
        println!("Attempting to connect to {}:{}", hostname, port);
        if let Ok(mut stream) = TcpStream::connect((hostname, port)).await {
            if let Ok(r) = ping(&mut stream, hostname, port).await {
                Ok(r)
            } else {
                Err(Error::from("Failed to ping the server for status"))
            }
        } else {
            Err(Error::from("Failed to open TCP stream"))
        }
    }

   pub fn output(&self) -> Result<impl Stream<Item = Result<LogOutput, bollard::errors::Error>>, Error> {
        #[cfg(unix)]
        let docker = if let Ok(d) = Docker::connect_with_socket_defaults() { d } else {
            return Err(Error::from("Couldn't connect to docker on default socket"));
        };

        let options = Some(LogsOptions::<String>{
            stdout: true,
            since: Utc::now().timestamp(),
            follow: true,
            ..Default::default()
        });

        Ok(docker.logs(&self.id, options))
    }

    pub fn clean_output(&self) -> Result<impl Stream<Item = Result<hyper::body::Bytes, bollard::errors::Error>>, Error> {
        match self.output() {
            Ok(l) => {
                Ok(l.filter_map(|msg| {
                    future::ready(match msg {
                        Ok(m) => {
                            // I think these are fine because they should always work
                            // TODO: Add lazy static crate?
                            // Add configuability for different matches?
                            let re1 = Regex::new(r"<.*>.*").unwrap();
                            let re2 = Regex::new(r"left the game").unwrap();
                            let re3 = Regex::new(r"joined the game").unwrap();
                            if re1.is_match(&m.to_string()) || re2.is_match(&m.to_string()) || re3.is_match(&m.to_string()) {
                                let text = m.to_string().split(" ").skip(3).fold(String::new(), |a, b| format!("{a} {b}")).trim_end_matches("\r\n").to_string();
                                Some(Ok(Bytes::from(text)))
                            } else {
                                None
                            }
                                }
                        Err(_) => None,
                    })
                }))
            },
            Err(e) => Err(e),
        }
    }
}

impl Unique<String> for Server {
    fn uuid(&self) -> String {
        self.name.clone()
    }
}

impl CloudSync<String> for Server {
    // TODO add this to the config later
    fn config() -> CLConfig {
        CLConfig {
            project_id: "mc-docker".to_string(),
            cred_path: "./firebase.json".to_string(),
            collection: "servers".to_string(),
        }
    }
}

/*
// IDK if I wanna do something like this or not. It makes implememntation tricky
// Trying to reimplememnt the builder pattern I've seen before here
pub struct ServerConfig {
    path: Option<String>, 
    port_arg: Option<u16>,
    ports: Option<Vec<u16>>, 
    version: Option<String>, 
    server_type: Option<String>, 
}

impl ServerConfig {
    pub fn new() -> ServerConfig {
        ServerConfig {
            path: None,
            port_arg: None,
            ports: None,
            version: None,
            server_type: None,
        }
    }

    pub fn path(&mut self, path: String) -> &ServerConfig {
        self.path = Some(path);
        self
    }

    pub fn port(&mut self, port: u16) -> &ServerConfig {
        self.port_arg = Some(port);
        self
    }

    pub fn ports(&mut self, ports: Vec<u16>) -> &ServerConfig {
        self.ports = Some(ports);
        self
    }

    pub fn version(&mut self, version: String) -> &ServerConfig {
        self.version = Some(version);
        self
    }

    pub fn server_type(&mut self, server_type: String) -> &ServerConfig {
        self.server_type = Some(server_type);
        self
    }
}
*/

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
