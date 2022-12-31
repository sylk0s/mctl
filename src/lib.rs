use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use server::Server;
use cloud::CloudSync;
use std::fs;
use serde::Deserialize;

pub mod server;
pub mod net;
pub mod status;
pub mod cloud;

pub type Servers = Arc<RwLock<HashMap<String, Server>>>;

pub async fn run() {
    let config = Config::get();

    let servers = load_from_cloud().await.unwrap();

    println!("Servers: {:?}", servers.write().await);
    net::start_ws(servers, config).await;    
}

pub async fn load_from_cloud() -> Option<Servers> {
    match Server::clget().await {
        Ok(cl_servers) => {
            let mut servers = HashMap::new();
            for server in cl_servers {
                servers.insert(server.name.clone(), server); 
            }
            Some(Arc::new(RwLock::new(servers)))
        },
        Err(_) => None,
    }
}

// change to use the $HOME
pub const CONF_PATH: &str = "/home/sylkos/.config/mc-docker";

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub fb_id: String,
    pub ws_port: u16,
    pub path: String,
    pub modules: Vec<String>,
}

impl Config {
    fn get() -> Config {
        let conf_file = fs::read_to_string(format!("{CONF_PATH}/config.toml")).expect("Failed to read config from fs");
        toml::from_str(&conf_file).expect("Error parsing config file from toml")
    }
}
