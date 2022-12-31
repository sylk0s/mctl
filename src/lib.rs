use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use server::Server;
use cloud::CloudSync;

pub mod server;
pub mod net;
pub mod status;
pub mod cloud;

pub type Servers = Arc<RwLock<HashMap<String, Server>>>;

pub async fn run() {
    let servers = load_from_cloud().await.unwrap();

    println!("Servers: {:?}", servers.write().await);
    net::start_ws(servers).await;    
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
