use futures::StreamExt;
use crate::server::Server;
use serde::{Serialize, Deserialize};
use craftping::sync::ping;
use std::net::TcpStream;
use warp::{
    http::{StatusCode, Response},
    reply::json,
    Reply, Rejection};
use crate::{Servers, Config};

type Result<T> = std::result::Result<T, Rejection>;
// Should probably reply with pong
pub async fn ping_handler() -> Result<impl Reply> {
    Ok(Response::builder().body("pong".to_string())) 
}

pub async fn start_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Started {id}");
    if let Some(s) = servers.write().await.get(&id) {
        if let Err(e) = s.start().await {
            return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(e))
        }
    } else {
        return Ok(Response::builder().status(StatusCode::NOT_FOUND).body("Server is not registered".to_string()))
    }
    Ok(Response::builder().body("Success!".to_string())) 
}

pub async fn stop_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Stopped {id}");
    if let Some(s) = servers.write().await.get(&id) {
        if let Err(e) = s.stop().await {
            return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(e))
        }
    } else {
        return Ok(Response::builder().status(StatusCode::NOT_FOUND).body("Server is not registered".to_string()))
    }
    Ok(Response::builder().body("Success!".to_string())) 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exec {
    args: Vec<String>,
}

pub async fn exec_handler(id: String, body: Exec, servers: Servers) -> Result<impl Reply> {
    println!("Executed {} on {id}", body.args.iter().fold(String::new(), |s, x| format!("{s} {x}")).trim());
    if let Some(s) = servers.write().await.get(&id) {
        if let Err(e) = s.send_command(body.args).await {
            return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(e))
        }
    } else {
        return Ok(Response::builder().status(StatusCode::NOT_FOUND).body("Server is not registered".to_string()))
    }
    Ok(Response::builder().body("Success!".to_string())) 
}

pub async fn output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting output from {id}");
    if let Some(s) = servers.write().await.get(&id) {
        if let Ok(o) = s.output() {
            Ok(
                Response::new(
                    hyper::Body::wrap_stream(
                        o.map(|item| 
                            match item {
                                Ok(out) => Ok(out.into_bytes()),
                                Err(e) => Err(e),
                            }
                        )
                    )
                )
            )
        // NOT a big fan of this method...
        // Normal thing won't work because it's not the same type
        } else {
            Err(warp::reject())
        }
    } else {
        Err(warp::reject())
    }
}

pub async fn clean_output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting clean output from {id}");
    if let Some(s) = servers.write().await.get(&id) {
        if let Ok(o) = s.clean_output() {
            Ok(
                Response::new(
                    hyper::Body::wrap_stream(o)
                )
            )
        // NOT a big fan of this method...
        // Normal thing won't work because it's not the same type
        } else {
            Err(warp::reject())
        }
    } else {
        Err(warp::reject())
    }
}

// I'm gonna neglect this one for a second because I wanna redo it later
pub async fn status_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Attempting to get status");
    let hostname = "localhost";
    let port = servers.write().await.get(&id).unwrap().port.clone();
    let mut stream = TcpStream::connect((hostname, port)).unwrap();
    let pong = ping(&mut stream, hostname, port).expect("Cannot ping server");
    Ok(json(&pong))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct New {
    id: String,
    path: Option<String>,
    port: Option<u16>,
    version: Option<String>,
    server_type: Option<String>,
}

pub async fn new_handler(body: New, servers: Servers, config: Config) -> Result<impl Reply> {
    println!("Creating new server...");
    let ports = servers.write().await.values().clone().map(|v| v.port).collect::<Vec<u16>>();
    match Server::new(body.id, body.path, body.port, Some(ports), body.version, body.server_type, config).await {
        Ok(s) => {
            servers.write().await.insert(s.name.clone(), s);
            Ok(Response::builder().body("Success!".to_string())) 
        },
        Err(e) => Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(e))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResponse {
    servers: Vec<String>,
}

pub async fn list_handler(servers: Servers) -> Result<impl Reply> {
    Ok(json(&ListResponse { servers: servers.write().await.keys().map(|a| a.to_owned()).collect() }))
}

// NOTE - Doesn't stop the container from running. I think I will leave that for the client to
// implemenet
pub async fn rm_handler(id: String, servers: Servers) -> Result<impl Reply> {
    servers.write().await.remove(&id);
    Ok(StatusCode::OK)
}
