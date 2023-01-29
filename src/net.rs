use std::convert::Infallible;
use warp::Filter;
use crate::{Servers, Config};
use crate::handlers::*;
use crate::error::handle_rejection;

// I wonder if theres anything I can do here the help the compile time of these.
pub async fn start_ws(servers: Servers, config: Config) {

    // Ping the server
    // /beep`
    let beep_route = warp::path!("beep")
        .and_then(beep_handler);

    // Execute a command on the server
    // /exec/{name} + json
    let exec_route = warp::path!("exec" / String)
        .and(warp::post())
        .and(warp::body::json())
        .and(with(servers.clone()))
        .and_then(exec_handler);

    // Start a server
    // /start/{name}
    let start_route = warp::path!("start" / String)
        .and(warp::put())
        .and(with(servers.clone()))
        .and_then(start_handler);

    // Stop a server
    // /stop/{name}
    let stop_route = warp::path!("stop" / String)
        .and(warp::put())
        .and(with(servers.clone()))
        .and_then(stop_handler);

    // Get the full output of a server
    // /fullout/{name}
    let full_output_route = warp::path!("fullout" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(full_output_handler);

    // Create a new server
    // /new + json
    let new_route = warp::path!("new")
        .and(warp::post())
        .and(warp::body::json())
        .and(with(servers.clone()))
        .and(with(config.clone()))
        .and_then(new_handler);

    // Create a backup of a server
    //let backup_route = warp::path("backup")
    // /backup/{id}/{_, /region/{x-y}}

    // Get the status of mc-docker
    // /status{ ,/{name} }
    let full_route = warp::path("statusaaa")
        .and(warp::path::end())
        .and(with(servers.clone()))
        .and_then(full_status_handler);

    let partial_route = warp::path("status")
        .and(warp::path::param())
        .and(with(servers.clone()))
        .and_then(partial_status_handler);
    
    // List all of the servers
    // /list
    let list_route = warp::path!("list")
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(list_handler);

    // Remove a server from memory
    // /rm/{name}
    let rm_route = warp::path!("rm" / String)
        .and(warp::delete())
        .and(with(servers.clone()))
        .and_then(rm_handler);

    // Gets a cleaned output from the server
    // /out/{name}
    let output_route = warp::path!("out" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(output_handler);

    let routes = beep_route
        .or(start_route)
        .or(exec_route)
        .or(stop_route)
        .or(full_output_route)
        .or(full_route)
        .or(partial_route)
        .or(new_route)
        .or(list_route)
        .or(rm_route)
        .or(output_route)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);

    println!("Everything loaded in, starting Web Server on port {} now...", config.ws_port);

    warp::serve(routes).run(([127, 0, 0, 1], config.ws_port)).await;
}

fn with<T>(items: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
    where T: Clone + Send 
{
    warp::any().map(move || items.clone())
}
