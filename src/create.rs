use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bollard::container::Config;

/*
 * Plans:
 *
 * - Be able to create a new server from some globally configurable defaults
 * - Be able to create a new server with arbitrary properties
 * - Be able to create a new server with a prederemined path and docker-compose.yml
 * - Be able to reload an existing server into the framework
 * - Be able to spin up a new server with a precreated save file
 * - Be able to spin up a new server with a precreated server file
 * - Be able to load weird stuff into docker like MC 1.12
 *
 */

// All this stuff is so that I can serialize docker-compose.yml into the config file

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ComposeYaml {
    version: String,
    services: Services, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Services {
    mc: MC,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MC {
    image: String,
    ports: Vec<String>,
    environment: Env,
    tty: bool,
    stdin_open: bool,
    restart: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
struct Env {
    eula: String,
    version: String,
}

impl ComposeYaml {
    pub fn test() {
        let yml = std::fs::read_to_string("docker-compose.yml").unwrap(); 
        let test_struct: ComposeYaml = serde_yaml::from_str(&yml).unwrap();
        println!("{:?}", test_struct);
    }

    fn to_config(&self) -> Config<String> {
        let mut ports = HashMap::new();
        // 
        let mut env = Vec::new();
        let mut vol = HashMap::new();
        Config {
            exposed_ports: Some(ports),
            tty: Some(true),
            open_stdin: Some(true),
            env: Some(env),
            image: Some(self.services.mc.image.clone()),
            volumes: Some(vol),
            ..Default::default()
        }
    }
}
