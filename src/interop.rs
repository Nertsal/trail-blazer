use crate::model::*;

use geng::prelude::*;

pub type ClientId = i64;

pub type ClientConnection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Setup(Setup),
    StartResolution(shared::SharedModel),
    FinishResolution(shared::SharedModel),
    PlayerCustomization(ClientId, PlayerCustomization),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Pong,
    SetCustomization(PlayerCustomization),
    Spectate,
    SubmitMove(PlayerMove),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setup {
    pub player_id: ClientId,
    pub model: shared::SharedModel,
}
