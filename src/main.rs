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

    //server.send_command(vec!["tellraw", "@a", "{\"text\":\"boop\"}"]).await.expect("It broken");

    //server.start().await.expect("Server failed to start");

    /*
    let mut logs = server.output();
    while let Some(msg) = logs.next().await {
        if let Ok(m) = msg {
            println!("Server msg: {}", m);
        }
    }
    */

    mc_docker::run().await;

    //mc_docker::create::ComposeYaml::test();

    Ok(())
}
