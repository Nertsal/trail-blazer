use super::*;

pub struct ClientConnection {
    pub id: ClientId,
    pub state: Arc<Mutex<ServerState>>,
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        let _client = state.clients.remove(&self.id).unwrap();
    }
}

impl geng::net::Receiver<ClientMessage> for ClientConnection {
    fn handle(&mut self, message: ClientMessage) {
        let mut state = self.state.lock().unwrap();
        let state: &mut ServerState = state.deref_mut();
        match message {
            ClientMessage::Pong => {
                let client = state
                    .clients
                    .get_mut(&self.id)
                    .expect("Sender not found for client");
                // client.sender.send(ServerMessage::Time(
                //     state.timer.elapsed().as_secs_f64() as f32
                // ));
                client.sender.send(ServerMessage::Ping);
            }
        }
    }
}
