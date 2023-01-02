use futures::StreamExt;
use crate::server::Server;
use serde::{Serialize, Deserialize};
use warp::{
    http::{StatusCode, Response},
    reply::json,
    Reply, Rejection, reject};
use crate::{Servers, Config};
use crate::error::*;

type Result<T> = std::result::Result<T, Rejection>;
// Should probably reply with pong
pub async fn beep_handler() -> Result<impl Reply> {
    Ok(Response::builder().body("boop".to_string())) 
}

pub async fn start_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Started {id}");
    if let Some(s) = servers.write().await.get(&id) {
        match s.start().await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(reject::custom(e))
        }
    } else {
        Err(reject::custom(NotRegistered { id }))
    }
}

pub async fn stop_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Stopped {id}");
    if let Some(s) = servers.write().await.get(&id) {
        match s.stop().await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(reject::custom(e))
        }
    } else {
        Err(reject::custom(NotRegistered { id }))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Exec {
    args: Vec<String>,
}

pub async fn exec_handler(id: String, body: Exec, servers: Servers) -> Result<impl Reply> {
    println!("Executed {} on {id}", body.args.iter().fold(String::new(), |s, x| format!("{s} {x}")).trim());
    if let Some(s) = servers.write().await.get(&id) {
        match s.send_command(body.args).await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(reject::custom(e))
        }
    } else {
        Err(reject::custom(NotRegistered { id }))
    }
}

pub async fn full_output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting output from {id}");
    if let Some(s) = servers.write().await.get(&id) {
        match s.output() {
            Ok(o) => Ok(Response::new(hyper::Body::wrap_stream(
                        o.map(|item| 
                            match item {
                                Ok(out) => Ok(out.into_bytes()),
                                Err(e) => Err(e),
                            })))),
            Err(e) => Err(reject::custom(e))
        }
    } else {
        Err(reject::custom(NotRegistered { id }))
    }
}

pub async fn output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting clean output from {id}");
    if let Some(s) = servers.write().await.get(&id) {
        match s.clean_output() {
            Ok(o) => Ok(Response::new(hyper::Body::wrap_stream(o))),
            Err(e) => Err(reject::custom(e))
        } 
    } else {
        Err(reject::custom(NotRegistered { id }))
    }
}

// I'm gonna neglect this one for a second because I wanna redo it later
pub async fn get_status(id: String, servers: Servers) -> std::result::Result<craftping::Response, Rejection> {
    println!("Attempting to get status of {id}");
    if let Some(s) = servers.write().await.get(&id) {
        match s.status().await {
            Ok(r) => Ok(r),
            Err(e) => Err(reject::custom(e))
        }
    } else {
        Err(reject::custom(NotRegistered {id}))
    }
}

pub async fn full_status_handler(servers: Servers) -> Result<impl Reply> {
    Ok(StatusCode::OK)
}

// I actually don't know what to do here
pub async fn partial_status_handler(id: String, servers: Servers) -> Result<impl Reply> {
    match get_status(id, servers).await {
        Ok(s) => Ok(json(&s)),
        Err(e) => Err(e)
    }
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
            Ok(StatusCode::OK) 
        },
        Err(e) => Err(reject::custom(e)) 
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
