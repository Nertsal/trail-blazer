use super::*;

impl Model {
    pub fn update(&mut self, delta_time: FTime) {}

    pub fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => {
                self.messages.push(ClientMessage::Pong);
            }
            ServerMessage::YourId(_id) => {}
            ServerMessage::Setup(_setup) => {}
        }
    }
}
