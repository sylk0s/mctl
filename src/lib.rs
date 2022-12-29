use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod server;
pub mod net;
pub mod create;
pub mod status;

pub type Servers = Arc<RwLock<HashMap<String, server::Server>>>;

pub async fn run() {
    net::start_ws().await;    
}
