use crate::model::*;

use geng::prelude::*;

pub type ClientId = i64;

pub type ClientConnection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Setup(Setup),
    StartResolution(shared::SharedModel),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Pong,
    SubmitMove(Vec<vec2<ICoord>>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setup {
    pub player_id: ClientId,
    pub model: shared::SharedModel,
}
