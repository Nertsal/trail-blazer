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
    pub queued_moves: HashMap<ClientId, PlayerMove>,
}

impl ServerState {
    pub const TICKS_PER_SECOND: f32 = 2.0;

    pub fn new() -> Self {
        let mut map = Map::new(vec2(14, 7));
        map.walls = vec![vec2(3, 0), vec2(-2, 0)];
        Self {
            timer: Timer::new(),
            next_id: 1,
            config: Config {},
            clients: HashMap::new(),
            model: SharedModel::new(map),
            queued_moves: HashMap::new(),
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
        self.model
            .players
            .insert(player_id, Player::new(player_id, Character::random(), pos));
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
                    for player in self.model.players.values_mut() {
                        player.submitted_move = self
                            .queued_moves
                            .get(&player.id)
                            .cloned()
                            .unwrap_or_default();
                    }
                    self.model.start_resolution();
                    for client in self.clients.values_mut() {
                        client
                            .sender
                            .send(ServerMessage::StartResolution(self.model.clone()));
                    }
                }
                GameEvent::FinishResolution => {
                    self.model.finish_resolution();
                    for client in self.clients.values_mut() {
                        client
                            .sender
                            .send(ServerMessage::FinishResolution(self.model.clone()));
                    }
                }
                GameEvent::MushroomsCollected(n) => {
                    for _ in 0..n {
                        self.model.spawn_mushroom();
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
            ClientMessage::SubmitMove(mov) => {
                self.queued_moves.insert(client_id, mov);
            }
        }
    }
}
