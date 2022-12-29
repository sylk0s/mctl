use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::json,
    Reply, Filter, Rejection};
use futures::StreamExt;
use crate::server::Server;
use serde::{Serialize, Deserialize};
use craftping::sync::ping;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;

type Result<T> = std::result::Result<T, Rejection>;
type Servers = Arc<RwLock<HashMap<String, Server>>>;

const PATH: &str = "/home/sylkos/servers";
const COMPOSE: &str = "/home/sylkos/docker-compose.yml";

pub async fn start_ws() {
    let servers: Servers = Arc::new(RwLock::new(HashMap::new()));

    let server = Server {
        name: "TEST".to_string(),
        path: "/home/sylkos/servers/test".to_string(),
        id: "249148a1229c".to_string(),
        port: 25565
    };

    let new = New {
        name: "test2".to_string(),
        path: Some("/home/sylkos/test2".to_string()),
        port: Some(25567),
    };

    servers.write().await.insert(server.name.clone(), server);

    status_handler("TEST".to_string(), servers.clone()).await.expect("aaa");

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

    // Get the output of a server in a stream
    let output_route = warp::path!("output" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(output_handler);

    // Create a new server
    let new_route = warp::path!("new")
        .and(warp::post())
        .and(warp::body::json())
        .and(with(servers.clone()))
        .and_then(new_handler);

    // Load a world file as a server
    //let load_route = warp::path("load")

    // Create a backup of a server
    //let backup_route = warp::path("backup")
    // /backup/{id}/{_, /region/{x-y}}

    // Get the status of mc-docker
    let status_route = warp::path!("status" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(status_handler);
    // /status/{id}/{_, /{stat}}

    let routes = ping_route
        .or(start_route)
        .or(exec_route)
        .or(stop_route)
        .or(output_route)
        .or(status_route)
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

async fn status_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Attempting to get status");
    let hostname = "localhost";
    let port = servers.write().await.get(&id).unwrap().port.clone();
    let mut stream = TcpStream::connect((hostname, port)).unwrap();
    let pong = ping(&mut stream, hostname, port).expect("Cannot ping server");
    Ok(json(&pong))
}

#[derive(Serialize, Deserialize, Debug)]
struct New {
    name: String,
    path: Option<String>,
    port: Option<u16>,
}

async fn new_handler(body: New, servers: Servers) -> Result<impl Reply> {
    let path_str = if let Some(p) = body.path {
                    p 
                } else {
                    format!("{PATH}/{}", body.name)
                };
    
    let path = Path::new(&path_str);

    if !path.exists() {
        std::fs::create_dir_all(path_str.clone()).expect("Error creating a new directory.");
    }

    let port = 12345;
    // if compose doesn't exists, assign the port in the call, if it doesn't exist, assign the next
    // available port above 25565

    let compose_str = format!("{path_str}/docker-compose.yml"); 
    let compose = Path::new(&compose_str);

    if !compose.exists() {
        fs::File::create(&compose_str).expect("Error creating docker compose");
        std::fs::copy(COMPOSE, compose_str).expect("Error copying default contents of docker compose"); 

        // read in compose and edit port
    }
    
    let output = Command::new("docker")
        .arg("compose")
        .arg("up")
        .arg("-d")
        .stdin(Stdio::piped())
        .output().unwrap();

    println!("{:?}", output);
    // parse output into ID
    let id = "".to_string();

    // add to Servers
    let server = Server {
        name: body.name,
        path: path_str,
        id,
        port,
    };

    servers.write().await.insert(server.name.clone(), server);
    Ok(StatusCode::OK)
}
