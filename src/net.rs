use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    http::StatusCode,
    Reply, Filter, Rejection};
use futures::{
    {FutureExt, StreamExt}, 
    future};
use crate::server::Server;
use serde::{Serialize, Deserialize};
use bollard::container::LogOutput;

#[derive(Debug, Clone)]
pub struct WsClient {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, WsClient>>>;
type Servers = Arc<RwLock<HashMap<String, Server>>>;

// Handles all incoming WS connections
// May move over to lib.rs if I decide to make the other calls use HTTPS
pub async fn start_ws() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let servers: Servers = Arc::new(RwLock::new(HashMap::new()));

    let server = Server {
        name: "TEST".to_string(),
        path: "bbb".to_string(),
        rcon: "ccc".to_string(),
        id: "249148a1229c".to_string(),
    };

    {
        servers.write().await.insert(server.name.clone(), server);
    }

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

    /*
    let output_route = warp::path!("output" / String)
        .and(warp::get())
        .and(with(servers.clone()))
        .and_then(output_handler);
        */

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

    // Add regular https stuff that doesn't require WS connection?
    let ws_route = warp::path("ws")
        .and(warp::ws())
        // figure out how to use these extra parameters
        // .and(warp::path::param())
        .and(with(clients.clone()))
        .and(with(servers.clone()))
        .and_then(ws_handler);

    let routes = ws_route
        .or(ping_route)
        .or(start_route)
        .or(exec_route)
        .or(stop_route)
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

// Tries to find a client with a matching 
async fn ws_handler(ws: warp::ws::Ws, /*id: String,*/ clients: Clients, servers: Servers) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| client_connection(socket, /*id,*/ clients)))
    /*
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
    */
}

// Should probably reply with pong
async fn ping_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK) 
}

pub async fn client_connection(ws: WebSocket, /*id: String,*/ clients: Clients/*, mut client: Client*/) {
    // black magic involving sender and such
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    // pipes the sender into the websocker's sender
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
                eprintln!("error sending websocket msg: {}", e);
        }
    }));

    // REDO later with some method once I figure out what to actually do with the WsClient
    let client = WsClient {
        user_id: 0,
        topics: vec![],
        sender: Some(client_sender),
    };

    // REDO also when I change this
    let id = client.user_id.to_string();
    clients.write().await.insert(id.clone(), client);

    println!("{} connected", id);

    // Handles messages from the client to the server
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients).await;
    }

    // Disconnected, removes client from registry
    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

// Write handles messages, I think it makes sense to have this as a wrapper that parses the message
// into a usable format and then have another function to handles specifically doing something on
// the server side with the client message.
async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Some(client) = clients.read().await.get(id).cloned() {
        if message == "aaa" {
            client.sender.unwrap().send(Ok(Message::text("a"))).expect("Message failed to send"); 
        } else if message == "bbb" {
            client.sender.unwrap().send(Ok(Message::text("b"))).expect("Message failed to send"); 
        } else {
            client.sender.unwrap().send(Ok(Message::text("c"))).expect("Message failed to send"); 
        }
    } else {
        println!("Client could not be found");
    }
}

async fn start_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Started {id}");
    servers.write().await.get(&id).unwrap().start().await.expect("Server failed to start");
    Ok(StatusCode::OK) 
}

fn print_return<T>(t: T) -> T where T: std::fmt::Debug { println!("{:?}", t); t }

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
    println!("Executed {} on {id}", body.args.iter().fold(String::new(), |s, x| format!("{s} {x}")));
    servers.write().await.get(&id).unwrap().send_command(body.args).await.expect("Server failed to execute command");
    Ok(StatusCode::OK) 
}

// Trying to implement a stream over http instead of having a websocket :3
/*
async fn output_handler(id: String, servers: Servers) -> Result<impl Reply> {
    println!("Getting putput from {id}");
    Ok(warp::reply::Response::new(hyper::Body::wrap_stream
                                  //::<StreamExt<Item = Result<LogOutput, bollard::errors::Error>>, String, bollard::errors::Error>
                                  (servers.write().await.get(&id).unwrap().output()
                                    .filter_map(|e|
                                                future::ready(if let LogOutput::StdOut { message: msg } = e.unwrap() {
                                                    Some(Ok(msg))
                                                } else {
                                                    None
                                                }))
                                                )))
        
}
*/

// Weird trait thing I was trying to do. May refactor the code later to include something like this
/*
#[async_trait]
trait ServerCommand {
    fn get_server(&self) -> Server {
        // use the servers hash to map local names to the docker ID
        unimplemented!();
    }

    fn get_id(&self) -> String;
    async fn exec<'a>(&self) -> Result<()>;
}

#[derive(Deserialize, Serialize, Debug)]
struct Start {
    id: String,
}

impl ServerCommand for Start {
    async fn exec(&self) -> Result<()> {
        self.get_server().start().await;
        Ok(())
    }

    fn get_id(&self) -> String {
        self.id 
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Stop {
    id: String,
}

impl ServerCommand for Stop {
    async fn exec(&self) -> Result<()> {
        Ok(()) 
    }

    fn get_id(&self) -> String {
        self.id
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Exec {
    id: String,
    cmd: Vec<String>,
}

impl ServerCommand for Exec {
    async fn exec(&self) -> Result<()> {
        Ok(())
    }

    fn get_id(&self) -> String {
        self.id
    }
}

async fn cmd_handler<T>(body: impl ServerCommand) -> Result<impl Reply> {
    if let Err(e) = body.exec().await {
        return Err(e)
    }
    Ok(StatusCode::OK)
}
*/
