use super::*;

pub struct SharedModel {
    pub map: Map,

    pub players: Vec<Player>,
    pub trails: Vec<PlayerTrail>,
}

impl SharedModel {
    pub fn new(map: Map) -> Self {
        Self {
            map,

            players: Vec::new(),
            trails: Vec::new(),
        }
    }
}
