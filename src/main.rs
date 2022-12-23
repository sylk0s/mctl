use mc_docker::server::Server;
use futures::StreamExt;


/*
 * Overview of how this works
 * 
 * One thread is used to handle recieving requests and acting accordingly
 * Each request spins up a new thread to respond to the incoming request
 * One thread is used per server running for an mcsp setup
 *
 * Overarching server manager runs a logical/physical server abstraction
 *
 * Things to figure out: What does each WS connection mean?
 * What is happening when a client connects
 * 1) Client is making an HTTPS request and gets a response
 *      OR
 *    Client connects over WS and "subscribes" to certain outputs
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

    //server.start().await.expect("Server failed to start");

    /*
    let mut logs = server.output();
    while let Some(msg) = logs.next().await {
        if let Ok(m) = msg {
            println!("Server MSG: {}", m);
        }
    }
    */
    mc_docker::run().await;

    Ok(())
}
