use geng::prelude::*;

pub type ClientConnection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Pong,
}
