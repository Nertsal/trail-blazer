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
        map.walls = vec![vec2(2, 0), vec2(-1, 0)];
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
        let mut position = None;
        for _ in 0..100 {
            let pos = self.model.map.random_position();
            if self.model.map.walls.contains(&pos)
                || self.model.players.values().any(|player| player.pos == pos)
            {
                continue;
            } else {
                position = Some(pos);
            }
        }
        if let Some(position) = position {
            self.model.players.insert(
                player_id,
                Player::new(player_id, PlayerCustomization::random(), position),
            );
        }

        for client in self.clients.values_mut() {
            client.sender.send(ServerMessage::Sync(self.model.clone()));
        }

        Setup {
            player_id,
            model: self.model.clone(),
        }
    }

    pub fn player_disconnect(&mut self, player_id: ClientId) {
        self.model.players.remove(&player_id);
        if self.model.players.is_empty() {
            self.model.new_game();
        }

        for client in self.clients.values_mut() {
            client.sender.send(ServerMessage::Sync(self.model.clone()));
        }
    }

    pub fn player_spectate(&mut self, client_id: ClientId) {
        self.model.players.remove(&client_id);
        if self.model.players.is_empty() {
            self.model.new_game();
        }
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.sender.send(ServerMessage::Sync(self.model.clone()));
        }
    }

    pub fn tick(&mut self) {
        let delta_time = FTime::new(ServerState::TICKS_PER_SECOND.recip());
        if self.model.players.is_empty() {
            if !matches!(self.model.phase, Phase::Starting { .. }) {
                self.model.new_game();
            }
        } else {
            for event in self.model.update(delta_time) {
                match event {
                    GameEvent::StartGame => {
                        self.model.start_game();
                        for client in self.clients.values_mut() {
                            client.sender.send(ServerMessage::Sync(self.model.clone()));
                        }
                    }
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

                        // Spawn mushrooms
                        let mushrooms = self.model.mushrooms.len();
                        let target = if thread_rng().gen_bool(0.2) { 2 } else { 1 };
                        for _ in mushrooms..target {
                            self.model.spawn_mushroom();
                        }

                        for client in self.clients.values_mut() {
                            client
                                .sender
                                .send(ServerMessage::FinishResolution(self.model.clone()));
                        }
                    }
                    GameEvent::ResultsOver => {
                        self.model.new_game();
                        for client in self.clients.values_mut() {
                            client
                                .sender
                                .send(ServerMessage::StartResolution(self.model.clone()));
                        }
                    }
                    _ => {}
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
            ClientMessage::SetCustomization(mut customization) => {
                if let Some(player) = self.model.players.get_mut(&client_id) {
                    customization.name = rustrict::Censor::from_str(&customization.name).collect();
                    player.customization = customization;

                    for client in self.clients.values_mut() {
                        client.sender.send(ServerMessage::PlayerCustomization(
                            player.id,
                            player.customization.clone(),
                        ));
                    }
                }
            }
            ClientMessage::Spectate => {
                self.player_spectate(client_id);
            }
            ClientMessage::SubmitMove(mov) => {
                self.queued_moves.insert(client_id, mov);
            }
        }
    }
}
