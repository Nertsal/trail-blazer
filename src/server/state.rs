use super::*;

pub struct Config {}

pub struct Client {
    pub sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

pub struct ServerState {
    pub timer: Timer,
    pub next_id: ClientId,
    pub config: Config,
    pub clients: HashMap<ClientId, Client>,
    pub map_size: vec2<ICoord>,
    pub model: Model,
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 1.0;

    pub fn new() -> Self {
        let map_size = vec2(10, 10);
        Self {
            timer: Timer::new(),
            next_id: 1,
            config: Config {},
            clients: HashMap::new(),
            map_size,
            model: Model::new(map_size),
        }
    }

    pub fn get_setup(&self) -> Setup {
        Setup {
            map_size: self.map_size,
        }
    }

    pub fn tick(&mut self) {
        let delta_time = FTime::new(ServerState::TICKS_PER_SECOND.recip());
        self.model.update(delta_time);
        self.model.messages.clear();
    }

    pub fn handle_message(&mut self, client_id: ClientId, message: ClientMessage) {
        match message {
            ClientMessage::Pong => {
                let client = self
                    .clients
                    .get_mut(&client_id)
                    .expect("Sender not found for client");
                // client.sender.send(ServerMessage::Time(
                //     state.timer.elapsed().as_secs_f64() as f32
                // ));
                client.sender.send(ServerMessage::Ping);
            }
        }
    }
}
