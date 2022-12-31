use std::convert::Infallible;
use warp::{
    http::StatusCode,
    reply::json,
    Reply, Filter, Rejection};
use futures::StreamExt;
use crate::server::Server;
use serde::{Serialize, Deserialize};
use craftping::sync::ping;
use std::net::TcpStream;
use crate::Servers;

type Result<T> = std::result::Result<T, Rejection>;

pub async fn start_ws(servers: Servers) {
    /*
    servers.write().await.insert(server.name.clone(), server);

    status_handler("TEST".to_string(), servers.clone()).await.expect("aaa");
    */

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

    // Create a backup of a server
    //let backup_route = warp::path("backup")
    // /backup/{id}/{_, /region/{x-y}}

    // Get the status of mc-docker
    let status_route = warp::path!("status" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(status_handler);
    // /status/{id}/{_, /{stat}}
    
    let list_route = warp::path!("list")
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(list_handler);

    let rm_route = warp::path!("rm" / String)
        .and(warp::delete())
        .and(with(servers.clone()))
        .and_then(rm_handler);

    let clean_route = warp::path!("cleanout" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(clean_output_handler);

    let routes = ping_route
        .or(start_route)
        .or(exec_route)
        .or(stop_route)
        .or(output_route)
        .or(status_route)
        .or(new_route)
        .or(list_route)
        .or(rm_route)
        .or(clean_route)
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

async fn clean_output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting clean output from {id}");
    Ok(warp::reply::Response::new(
            hyper::Body::wrap_stream(
                servers.write().await.get(&id).unwrap().clean_output())))
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
    id: String,
    path: Option<String>,
    port: Option<u16>,
    version: Option<String>,
    server_type: Option<String>,
}

async fn new_handler(body: New, servers: Servers) -> Result<impl Reply> {
    println!("Creating new server...");
    let ports = servers.write().await.values().clone().map(|v| v.port).collect::<Vec<u16>>();
    let server = Server::new(body.id, body.path, body.port, Some(ports), body.version, body.server_type).await;
    servers.write().await.insert(server.name.clone(), server);
    Ok(StatusCode::OK)
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResponse {
    servers: Vec<String>,
}

async fn list_handler(servers: Servers) -> Result<impl Reply> {
    Ok(json(&ListResponse { servers: servers.write().await.keys().map(|a| a.to_owned()).collect() }))
}

// NOTE - Doesn't stop the container from running. I think I will leave that for the client to
// implemenet
async fn rm_handler(id: String, servers: Servers) -> Result<impl Reply> {
    servers.write().await.remove(&id);
    Ok(StatusCode::OK)
}
