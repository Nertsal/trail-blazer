use super::*;

use geng::prelude::itertools::Itertools;

pub const TIME_PER_PLAN: f32 = 5.0;
pub const TIME_PER_MOVE: f32 = 0.5;

#[derive(Debug, Clone)]
pub enum GameEvent {
    StartResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Planning { time_left: FTime },
    Resolution { next_move_in: FTime },
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
            Phase::Resolution { next_move_in } => {
                *next_move_in -= delta_time;
                if *next_move_in <= FTime::ZERO {
                    let next = *next_move_in;
                    if self.resolve_next_move() {
                        self.phase = Phase::Resolution {
                            next_move_in: next + FTime::new(TIME_PER_MOVE),
                        };
                    } else {
                        for player in self.players.values_mut() {
                            player.submitted_move.clear();
                        }
                        self.phase = Phase::Planning {
                            time_left: next + FTime::new(TIME_PER_PLAN),
                        }
                    }
                }
            }
        }
        events
    }

    pub fn start_resolution(&mut self) {
        let Phase::Planning { .. } = self.phase else {
            return;
        };

        // Validate paths
        let mut invalid = Vec::new();
        for player in self.players.values() {
            if !self.validate_path(player.id, &player.submitted_move) {
                invalid.push(player.id);
            }
        }

        for player in self.players.values_mut() {
            if invalid.contains(&player.id) {
                player.resolution_speed_left = 0;
                player.submitted_move.clear();
            } else {
                player.resolution_speed_left = player.speed;
            }
        }
        self.trails.clear();
        self.phase = Phase::Resolution {
            next_move_in: FTime::ZERO,
        };
    }

    /// Resolves the next batch of player moves,
    /// returns true if more moves need to be resolved
    /// and false if all moves are resolved.
    pub fn resolve_next_move(&mut self) -> bool {
        let resolving_speed = self
            .players
            .values()
            .map(|player| player.resolution_speed_left)
            .max()
            .unwrap_or(0);
        if resolving_speed == 0 {
            return false;
        }

        let mut target_moves: HashMap<vec2<ICoord>, Vec<ClientId>> = HashMap::new();
        self.players
            .values()
            .filter(|player| player.resolution_speed_left == resolving_speed)
            .for_each(|player| {
                if let Some(&pos) = player
                    .submitted_move
                    .get(1 + player.speed - player.resolution_speed_left)
                {
                    target_moves.entry(pos).or_default().push(player.id);
                }
            });

        if target_moves.is_empty() {
            return false;
        }

        // Check for bounces (multiple players moving into the same tile)
        for (target, players) in target_moves {
            if players.len() <= 1 {
                // Just move the player - check for other collisions
                for player in players {
                    if let Some(player) = self.players.get_mut(&player) {
                        self.trails.push(PlayerTrail {
                            player: player.id,
                            pos: player.pos,
                        });
                        player.pos = target;
                        player.resolution_speed_left -= 1;
                    }
                }
            } else {
                // Bounce all players
                for player in players {
                    if let Some(player) = self.players.get_mut(&player) {
                        player.resolution_speed_left = 0;
                    }
                }
            }
        }

        // Double check resolution speed
        for player in self.players.values_mut() {
            player.resolution_speed_left = player.resolution_speed_left.min(resolving_speed - 1);
        }

        true
    }

    pub fn validate_path(&self, player_id: ClientId, path: &[vec2<ICoord>]) -> bool {
        let Some(player) = self.players.get(&player_id) else {
            return false;
        };

        if path.len() > player.speed + 1 {
            return false; // Path exceed player's speed
        }

        if path.first() != Some(&player.pos) {
            return false; // Path does not start at player's position
        }

        // Check adjacency, walls, and bounds
        for (&from, &to) in path.iter().tuple_windows() {
            if self.map.walls.contains(&to) || !are_adjacent(from, to) || !self.map.is_in_bounds(to)
            {
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
