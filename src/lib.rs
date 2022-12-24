use std::{
    collections::HashMap,
    sync::{Arc, Mutex}};
use server::Server;

pub mod server;
pub mod net;
pub mod create;

type Servers = Arc<Mutex<HashMap<String, Server>>>;

pub async fn run() {
    let servers: Servers = Arc::new(Mutex::new(HashMap::new()));

    //ws::start_ws().await;    
}
