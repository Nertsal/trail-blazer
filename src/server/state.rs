use super::*;

use crate::model::shared::*;

pub struct Config {}

pub struct Client {
    pub sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

pub struct ServerState {
    pub timer: Timer,
    pub next_id: ClientId,
    pub config: Config,
    pub clients: HashMap<ClientId, Client>,
    pub model: SharedModel,
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 1.0;

    pub fn new() -> Self {
        let mut map = Map::new(vec2(14, 7));
        map.walls = vec![vec2(3, 0), vec2(-2, 0)];
        Self {
            timer: Timer::new(),
            next_id: 1,
            config: Config {},
            clients: HashMap::new(),
            model: SharedModel::new(map),
        }
    }

    pub fn new_player(&mut self, player_id: ClientId) -> Setup {
        // NOTE: can infinite loop
        let pos = loop {
            let pos = self.model.map.random_position();
            if self.model.map.walls.contains(&pos)
                || self.model.players.values().any(|player| player.pos == pos)
            {
                continue;
            } else {
                break pos;
            }
        };
        self.model.players.insert(
            player_id,
            Player {
                id: player_id,
                character: Character::random(),
                pos,
                speed: 5,
                submitted_move: vec![],
            },
        );
        Setup {
            player_id,
            model: self.model.clone(),
        }
    }

    pub fn tick(&mut self) {
        let delta_time = FTime::new(ServerState::TICKS_PER_SECOND.recip());
        for event in self.model.update(delta_time) {
            match event {
                GameEvent::StartResolution => {
                    self.model.start_resolution();
                    for client in self.clients.values_mut() {
                        client
                            .sender
                            .send(ServerMessage::StartResolution(self.model.clone()));
                    }
                }
            }
        }
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
            ClientMessage::SubmitMove(path) => {
                if let Phase::Planning { .. } = self.model.phase
                    && self.model.validate_path(client_id, &path)
                    && let Some(player) = self.model.players.get_mut(&client_id)
                {
                    player.submitted_move = path;
                }
            }
        }
    }
}
