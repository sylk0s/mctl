pub mod server;
pub mod ws;

pub async fn run() {
    ws::start_ws().await;    
}
