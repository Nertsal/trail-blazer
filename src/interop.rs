use crate::model::*;

use geng::prelude::*;

pub type ClientId = i64;

pub type ClientConnection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Setup(Setup),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setup {
    pub player_id: ClientId,
    pub model: shared::SharedModel,
}
