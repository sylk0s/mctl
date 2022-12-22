use mc_docker::server::Server;


/*
 * Overview of how this works
 * 
 * One thread is used to handle recieving requests and acting accordingly
 * Each request spins up a new thread to respond to the incoming request
 * One thread is used per server running for an mcsp setup
 *
 * Overarching server manager runs a logical/physical server abstraction
 *
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let server = Server {
        name: "aaa".to_string(),
        path: "bbb".to_string(),
        rcon: "ccc".to_string(),
        id: "6939b3fa9ce3".to_string(),
    };

    //server.send_command(vec!["tellraw", "@a", "{\"text\":\"boop\"}"]).await.expect("It broken");

    // 

    Ok(())
}
