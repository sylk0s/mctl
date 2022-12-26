pub mod server;
pub mod net;
pub mod create;

pub async fn run() {
    net::start_ws().await;    
}
