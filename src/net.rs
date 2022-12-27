use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    Reply, Filter, Rejection};
use futures::StreamExt;
use crate::server::Server;
use serde::{Serialize, Deserialize};

type Result<T> = std::result::Result<T, Rejection>;
type Servers = Arc<RwLock<HashMap<String, Server>>>;

pub async fn start_ws() {
    let servers: Servers = Arc::new(RwLock::new(HashMap::new()));

    let server = Server {
        name: "TEST".to_string(),
        path: "bbb".to_string(),
        rcon: "ccc".to_string(),
        id: "249148a1229c".to_string(),
    };

    servers.write().await.insert(server.name.clone(), server);

    // Ping the server
    let ping_route = warp::path!("beep")
        .and_then(ping_handler);

    // Execute a command on the server
    let exec_route = warp::path!("exec" / String)
        .and(warp::post())
        .and(warp::body::json())
        .and(with(servers.clone()))
        .and_then(exec_handler);

    // Start a server
    let start_route = warp::path!("start" / String)
        .and(warp::put())
        .and(with(servers.clone()))
        .and_then(start_handler);

    // Stop a server
    let stop_route = warp::path!("stop" / String)
        .and(warp::put())
        .and(with(servers.clone()))
        .and_then(stop_handler);

    let output_route = warp::path!("output" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(output_handler);

    // Create a new server
    //let new_route = warp::path("new")

    // Load a world file as a server
    //let load_route = warp::path("load")

    // Create a backup of a server
    //let backup_route = warp::path("backup")
    // /backup/{id}/{_, /region/{x-y}}

    // Get the status of mc-docker
    //let status_route = warp::path("status") 
    // /status/{id}/{_, /{stat}}

    let routes = ping_route
        .or(start_route)
        .or(exec_route)
        .or(stop_route)
        .or(output_route)
        .with(warp::cors().allow_any_origin());

    println!("Everything loaded in, starting Web Server now...");

    // fix hardcoded stuff with config file later
    warp::serve(routes).run(([127, 0, 0, 1], 7955)).await;
}

fn with<T>(items: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
    where T: Clone + Send 
{
    warp::any().map(move || items.clone())
}

// Should probably reply with pong
async fn ping_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK) 
}

async fn start_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Started {id}");
    servers.write().await.get(&id).unwrap().start().await.expect("Server failed to start");
    Ok(StatusCode::OK) 
}

async fn stop_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Stopped {id}");
    servers.write().await.get(&id).unwrap().stop().await.expect("Server failed to stop");
    Ok(StatusCode::OK) 
}

#[derive(Serialize, Deserialize, Debug)]
struct Exec {
    args: Vec<String>,
}

async fn exec_handler(id: String, body: Exec, servers: Servers) -> Result<impl Reply> {
    println!("Executed {} on {id}", body.args.iter().fold(String::new(), |s, x| format!("{s} {x}")).trim());
    servers.write().await.get(&id).unwrap().send_command(body.args).await.expect("Server failed to execute command");
    Ok(StatusCode::OK) 
}

async fn output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting output from {id}");
    Ok(warp::reply::Response::new(
            hyper::Body::wrap_stream(
                servers.write().await.get(&id).unwrap().output().map(|item| 
                                                                     match item {
                                                                        Ok(out) => Ok(out.into_bytes()),
                                                                        Err(e) => Err(e),
                                                                     }))))
}
