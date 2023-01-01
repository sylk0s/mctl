use std::convert::Infallible;
use warp::Filter;
use crate::{Servers, Config};
use crate::handlers::*;


pub async fn start_ws(servers: Servers, config: Config) {

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
        .and(with(config.clone()))
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

    println!("Everything loaded in, starting Web Server on port {} now...", config.ws_port);

    warp::serve(routes).run(([127, 0, 0, 1], config.ws_port)).await;
}

fn with<T>(items: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
    where T: Clone + Send 
{
    warp::any().map(move || items.clone())
}
