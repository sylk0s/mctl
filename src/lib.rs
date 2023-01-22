use std::collections::HashMap; 
use std::sync::Arc; 
use tokio::sync::RwLock; 
use server::Server; 
use cloud::CloudSync;
use std::fs; 
use serde::Deserialize; 
use std::process::Command; 

pub mod server;
pub mod net;
pub mod status;
pub mod cloud;
pub mod handlers;
pub mod error;

pub type Servers = Arc<RwLock<HashMap<String, Server>>>;

pub async fn run() {
    // fix this
    if let Some(config) = Config::get() {
        let servers = load_from_cloud().await;

        println!("Servers: {:?}", servers.write().await);

        load_modules(config.clone()).await;
        net::start_ws(servers, config).await;  
    } else {
        println!("Some error getting the config file from the filepath, please fix this and run mc-docker again :3");
    }
}

/// Returns the list of servers from the cloud, if none are found, creates an empty
/// list and warns the user
pub async fn load_from_cloud() -> Servers {
    Arc::new(RwLock::new(match Server::clget().await {
        Ok(cl_servers) => {
            let mut servers = HashMap::new();
            for server in cl_servers {
                servers.insert(server.name.clone(), server); 
            }
            servers
        },
        Err(_) => {
            println!("Some error occured syncing error from the cloud, starting mc-docker without any loaded servers");
            HashMap::new()
        },
    }))
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
    fn get() -> Option<Config> {
        if let Ok(file) = fs::read_to_string(format!("{CONF_PATH}/config.toml")) {
            Some(toml::from_str(&file).expect("Error parsing config file from toml"))
        } else {
            // touch config file
            None
        }
    }
}

async fn load_modules(config: Config) {
    for module in config.modules {
        tokio::spawn(async move {
            // change this so we can specific a module path?
            let _m = Command::new(module)
                .spawn().unwrap();
        });
    }
}
