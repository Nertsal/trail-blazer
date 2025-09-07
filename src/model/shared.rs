use super::*;

use geng::prelude::itertools::Itertools;

pub const TIME_PER_PLAN: f32 = 5.0;

#[derive(Debug, Clone)]
pub enum GameEvent {
    StartResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Planning { time_left: FTime },
    Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedModel {
    pub map: Map,

    pub phase: Phase,
    pub players: HashMap<ClientId, Player>,
    pub trails: Vec<PlayerTrail>,
}

impl SharedModel {
    pub fn new(map: Map) -> Self {
        Self {
            map,

            phase: Phase::Planning {
                time_left: FTime::new(TIME_PER_PLAN),
            },
            players: HashMap::new(),
            trails: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: FTime) -> Vec<GameEvent> {
        let mut events = Vec::new();
        match &mut self.phase {
            Phase::Planning { time_left } => {
                *time_left -= delta_time;
                if *time_left <= FTime::ZERO {
                    events.push(GameEvent::StartResolution);
                }
            }
            Phase::Resolution => {}
        }
        events
    }

    pub fn start_resolution(&mut self) {
        let Phase::Planning { .. } = self.phase else {
            return;
        };
        // self.phase = Phase::Resolution;
        for player in self.players.values_mut() {
            if let Some(&pos) = player.submitted_move.last() {
                player.pos = pos;
            }
            player.submitted_move.clear();
        }
        self.phase = Phase::Planning {
            time_left: FTime::new(TIME_PER_PLAN),
        };
    }

    pub fn validate_path(&self, player_id: ClientId, path: &[vec2<ICoord>]) -> bool {
        let Some(player) = self.players.get(&player_id) else {
            return false;
        };

        if path.len() as ICoord > player.speed + 1 {
            return false; // Path exceed player's speed
        }

        if path.first() != Some(&player.pos) {
            return false; // Path does not start at player's position
        }

        // Check adjacency and walls
        for (from, to) in path.iter().tuple_windows() {
            if self.map.walls.contains(to) || !are_adjacent(*from, *to) {
                return false;
            }
        }

        true
    }
}

pub fn are_adjacent(a: vec2<ICoord>, b: vec2<ICoord>) -> bool {
    let d = b - a;
    d.x.abs() + d.y.abs() == 1
}
