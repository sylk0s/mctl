use mc_docker::server::Server;

/*
 *
 * mc-docker:
 *
 * A webserver "wrapper" for minecraft servers running in docker
 * Uses HTTP for most basic calls
 * Uses Web Sockets for piping server output and (eventually) having an interactive session
 * Stores server data in firestore
 *
 * TODO:
 * - Implement the HTTP endpoints for most things
 * - Figure out how applications with auth with mc-docker
 * - Play with the docker container to figure out how it creates servers
 * - Finish properly implementing compose -> config
 * - Implement the cloud sync thingy from sylk-bot
 * - determine if I wanna do the output with pub/sub or the dumb approach
 * - Add configs to everything
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
    //mc_docker::run().await;

    mc_docker::create::ComposeYaml::test();

    Ok(())
}
