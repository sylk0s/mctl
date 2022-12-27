pub mod server;
pub mod net;
pub mod create;
pub mod status;

pub async fn run() {
    net::start_ws().await;    
}
