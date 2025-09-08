use super::*;

use geng::prelude::itertools::Itertools;

pub const TIME_PER_PLAN: f32 = 5.0;
pub const TIME_PER_MOVE: f32 = 0.5;
pub const SPRINT_COOLDOWN: Turns = 3;
pub const TELEPORT_COOLDOWN: Turns = 3;
pub const TELEPORT_SPEED: usize = 5;
pub const THROW_SPEED: usize = 5;

#[derive(Debug, Clone)]
pub enum GameEvent {
    StartResolution,
    FinishResolution,
    MushroomsCollected(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Planning { time_left: FTime },
    Resolution { next_move_in: FTime },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mushroom {
    pub position: vec2<ICoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedModel {
    pub map: Map,
    pub turns_max: Turns,

    pub turn_current: Turns,
    pub phase: Phase,

    pub base: vec2<ICoord>,
    pub players: HashMap<ClientId, Player>,
    pub mushrooms: Vec<Mushroom>,
    pub trails: Vec<PlayerTrail>,
}

impl SharedModel {
    pub fn new(map: Map) -> Self {
        let mut model = Self {
            turn_current: 1,
            phase: Phase::Planning {
                time_left: FTime::new(TIME_PER_PLAN),
            },

            base: map.bounds.center(),
            players: HashMap::new(),
            mushrooms: Vec::new(),
            trails: Vec::new(),

            turns_max: 30,
            map,
        };
        model.spawn_mushroom();
        model
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

                    let (es, resolve) = self.resolve_next_move();
                    events.extend(es);

                    if resolve {
                        self.phase = Phase::Resolution {
                            next_move_in: next + FTime::new(TIME_PER_MOVE),
                        };
                    } else {
                        events.push(GameEvent::FinishResolution);
                    }
                }
            }
        }
        events
    }

    pub fn spawn_mushroom(&mut self) {
        let mut position = None;
        for _ in 0..10 {
            let pos = self.map.random_position();
            if distance(self.base, pos) <= 2
                || self.map.walls.contains(&pos)
                || self
                    .players
                    .values()
                    .any(|player| distance(player.pos, pos) <= 2)
            {
                continue;
            }

            position = Some(pos);
        }

        let Some(position) = position else { return };
        self.mushrooms.push(Mushroom { position });
    }

    pub fn finish_resolution(&mut self) {
        for player in self.players.values_mut() {
            // Clear move
            std::mem::take(&mut player.submitted_move);

            // Update stun
            if let Some(stun) = &mut player.stunned_duration {
                *stun -= 1;
                if *stun < 0 {
                    player.stunned_duration = None;
                }
            }

            // Update cooldowns
            player.cooldown_sprint -= 1;
            player.cooldown_teleport -= 1;
        }

        self.turn_current += 1;
        self.phase = Phase::Planning {
            time_left: FTime::new(TIME_PER_PLAN),
        }
    }

    pub fn start_resolution(&mut self) {
        let Phase::Planning { .. } = self.phase else {
            return;
        };

        // Validate paths
        let mut invalid = Vec::new();
        for player in self.players.values() {
            if !self.validate_move(player.id, &player.submitted_move) {
                invalid.push(player.id);
            }
        }

        // Update players
        for player in self.players.values_mut() {
            if player.stunned_duration.is_some() || invalid.contains(&player.id) {
                player.resolution_speed_left = 0;
                std::mem::take(&mut player.submitted_move);
            } else {
                let speed = match player.submitted_move {
                    PlayerMove::Normal { sprint, .. } => {
                        if sprint {
                            player.cooldown_sprint = SPRINT_COOLDOWN;
                        }
                        player.speed(sprint)
                    }
                    PlayerMove::TeleportChanneling => {
                        player.cooldown_teleport = TELEPORT_COOLDOWN;
                        TELEPORT_SPEED
                    }
                    PlayerMove::TeleportActivate { .. } => TELEPORT_SPEED,
                    PlayerMove::Throw { .. } => THROW_SPEED,
                };
                player.resolution_speed_max = speed;
                player.resolution_speed_left = speed;
            }
        }
        self.trails.clear();
        self.phase = Phase::Resolution {
            next_move_in: FTime::ZERO,
        };
    }

    /// Resolves the next batch of moves,
    /// returns true if more moves need to be resolved
    /// and false if all moves are resolved.
    pub fn resolve_next_move(&mut self) -> (Vec<GameEvent>, bool) {
        let mut events = Vec::new();

        let resolving_speed = self
            .players
            .values()
            .map(|player| player.resolution_speed_left)
            .max()
            .unwrap_or(0);
        if resolving_speed == 0 {
            return (events, false);
        }

        let mut target_moves: HashMap<vec2<ICoord>, Vec<ClientId>> = HashMap::new();
        for player in self.players.values_mut() {
            match &player.submitted_move {
                PlayerMove::Normal { path, .. } => {
                    if player.resolution_speed_left == resolving_speed
                        && let Some(&pos) =
                            path.get(1 + player.resolution_speed_max - player.resolution_speed_left)
                    {
                        target_moves.entry(pos).or_default().push(player.id);
                    }
                }
                PlayerMove::TeleportChanneling => {
                    if player.resolution_speed_left == player.resolution_speed_max {
                        player.is_channeling = true;
                    }
                }
                PlayerMove::TeleportActivate { teleport_to } => {
                    if player.resolution_speed_left == player.resolution_speed_max {
                        // Teleport on the first move
                        target_moves
                            .entry(*teleport_to)
                            .or_default()
                            .push(player.id);
                    }
                }
                PlayerMove::Throw { direction } => {}
            }
        }

        if target_moves.is_empty() {
            return (events, false);
        }

        // Check for bounces (multiple players moving into the same tile)
        for (target, players) in target_moves {
            if players.len() <= 1 {
                // Just move the player - check for other collisions
                for player in players {
                    // Check collisions
                    if self.players.values().any(|player| player.pos == target)
                        || self.trails.iter().any(|trail| trail.pos == target)
                    {
                        self.stun_player(player, 1);
                        continue;
                    }

                    // Move
                    if let Some(player) = self.players.get_mut(&player) {
                        if let Some(shroom_i) = self
                            .mushrooms
                            .iter()
                            .position(|shroom| shroom.position == target)
                        {
                            // Collect mushroom
                            player.mushrooms += 1;
                            self.mushrooms.swap_remove(shroom_i);
                        }

                        if self.base == target {
                            // Submit resources to base
                            player.collected_mushrooms += player.mushrooms;
                            events.push(GameEvent::MushroomsCollected(player.mushrooms));
                            player.mushrooms = 0;
                        }

                        if let PlayerMove::Normal { path, .. } = &player.submitted_move {
                            let connection_from = path
                                .iter()
                                .position(|&pos| pos == player.pos)
                                .and_then(|i| i.checked_sub(1))
                                .and_then(|i| path.get(i).copied());
                            self.trails.push(PlayerTrail {
                                player: player.id,
                                pos: player.pos,
                                connection_from,
                                connection_to: target,
                            });
                        }
                        player.pos = target;
                        player.resolution_speed_left -= 1;
                    }
                }
            } else {
                // Bounce all players
                for player in players {
                    self.stun_player(player, 1);
                }
            }
        }

        // Double check resolution speed
        for player in self.players.values_mut() {
            player.resolution_speed_left = player.resolution_speed_left.min(resolving_speed - 1);
        }

        (events, true)
    }

    pub fn stun_player(&mut self, player_id: ClientId, duration: Turns) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };

        player.resolution_speed_left = 0;
        player.is_channeling = false;
        player.stunned_duration = Some(duration);

        // Drop mushroom
        if player.mushrooms > 0
            && let PlayerMove::Normal { path, .. } = &player.submitted_move
            && let Some(&start_pos) = path.first()
            && start_pos != player.pos
        {
            player.mushrooms -= 1;
            self.mushrooms.push(Mushroom {
                position: start_pos,
            });
        }

        std::mem::take(&mut player.submitted_move);
    }

    pub fn validate_move(&self, player_id: ClientId, player_move: &PlayerMove) -> bool {
        let Some(player) = self.players.get(&player_id) else {
            return false;
        };

        match player_move {
            PlayerMove::Normal { path, sprint } => self.validate_path(player_id, path, *sprint),
            PlayerMove::TeleportChanneling => player.cooldown_teleport <= 0,
            PlayerMove::TeleportActivate { teleport_to } => {
                player.is_channeling && distance(player.pos, *teleport_to) <= 3
            }
            PlayerMove::Throw { direction } => {
                player.mushrooms > 0 && (direction.x == 0 || direction.y == 0)
            }
        }
    }

    pub fn validate_path(
        &self,
        player_id: ClientId,
        path: &[vec2<ICoord>],
        mut sprint: bool,
    ) -> bool {
        let Some(player) = self.players.get(&player_id) else {
            return false;
        };

        if player.cooldown_sprint > 0 {
            sprint = false;
        }

        if path.len() > player.speed(sprint) + 1 {
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

pub fn distance(a: vec2<ICoord>, b: vec2<ICoord>) -> ICoord {
    let d = b - a;
    d.x.abs() + d.y.abs()
}
