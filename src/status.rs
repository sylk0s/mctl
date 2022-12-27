use serde::Deserialize;
use std::{
    io::{prelude::*, BufReader},
    net::TcpStream};


/*
{
    "version": {
        "name": "1.19",
        "protocol": 759
    },
    "players": {
        "max": 100,
        "online": 5,
        "sample": [
            {
                "name": "thinkofdeath",
                "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
            }
        ]
    },
    "description": {
        "text": "Hello world"
    },
    "favicon": "data:image/png;base64,<data>",
    "previewsChat": true,
    "enforcesSecureChat": true,
}
 */

#[derive(Deserialize, Debug)]
pub struct Status {
    version: Version,
    players: Players,
    description: Motd,
    favicon: String,
}

#[derive(Deserialize, Debug)]
struct Version {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Players {
    max: u32,
    online: u32,
    sample: Vec<Player>
}

#[derive(Deserialize, Debug)]
struct Player {
    name: String,
    id: String,
}

#[derive(Deserialize, Debug)]
struct Motd {
    text: String
}

impl Status {
    pub fn request(ip: String) -> Result<Status, String> {
        if let Ok(mut stream) = TcpStream::connect(ip.clone()) {
            println!("Connected to the server");
           
            let initial_msg = format!("\\x00\\x00{ip}\\x01");
            let status_req = r"\x00";

            stream.write_all(initial_msg.as_bytes()).unwrap();
            stream.write_all(status_req.as_bytes()).unwrap();

            let buf_reader = BufReader::new(&mut stream);
            let status_request: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            // implement ping with unix time
            println!("Status: {:#?}", status_request);

        } else {
            println!("Couldn't connect to the server at {ip}")
        }
        Err("Failed to connect to the server".to_string())
    }

    fn from(s: String) -> Status {
        unimplemented!(); 
    }

    pub fn build() {

    }
}
