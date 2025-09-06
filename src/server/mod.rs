use crate::{interop::*, model::*};

use geng::prelude::*;

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }
}

impl geng::net::server::App for App {
    type Client = ClientConnection;

    type ServerMessage = ServerMessage;

    type ClientMessage = ClientMessage;

    fn connect(&mut self, sender: Box<dyn geng::net::Sender<Self::ServerMessage>>) -> Self::Client {
        todo!()
    }
}

impl geng::net::Receiver<ClientMessage> for ClientConnection {
    fn handle(&mut self, message: ClientMessage) {
        todo!()
    }
}
