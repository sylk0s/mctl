use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    http::StatusCode,
    Reply, Filter, Rejection};
use futures::{FutureExt, StreamExt};

#[derive(Debug, Clone)]
pub struct WsClient {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, WsClient>>>;

// Handles all incoming WS connections
// May move over to lib.rs if I decide to make the other calls use HTTPS
pub async fn start_ws() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let ping_route = warp::path!("ping")
        .and_then(ping_handler);

    // Add regular https stuff that doesn't require WS connection?
    let ws_route = warp::path("ws")
        .and(warp::ws())
        // figure out how to use these extra parameters
        // .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(ws_handler);

    let routes = ws_route
        .or(ping_route)
        .with(warp::cors().allow_any_origin());

    // fix hardcoded stuff with config file later
    warp::serve(routes).run(([127, 0, 0, 1], 7955)).await;
}

// clones the clients into the arg for the handler?
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

// Tries to find a client with a matching 
async fn ws_handler(ws: warp::ws::Ws, /*id: String,*/ clients: Clients) -> Result<impl Reply> {
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
