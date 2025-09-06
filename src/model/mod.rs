pub mod client;
pub mod shared;

use geng::prelude::*;
use geng_utils::conversions::*;

pub type ICoord = i64;
pub type FCoord = R32;
pub type Turns = u64;
pub type FTime = R32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub bounds: Aabb2<ICoord>,
    pub cell_size: vec2<FCoord>,
    pub walls: Vec<vec2<ICoord>>,
}

impl Map {
    pub fn new(size: vec2<ICoord>) -> Self {
        Self {
            bounds: Aabb2::from_corners(-size / 2 - size.map(|x| x % 2 - 1), size / 2),
            cell_size: vec2::splat(FCoord::ONE),
            walls: Vec::new(),
        }
    }

    pub fn to_world(&self, pos: vec2<ICoord>) -> vec2<FCoord> {
        self.cell_size * pos.as_r32()
    }

    pub fn world_bounds(&self) -> Aabb2<FCoord> {
        Aabb2::from_corners(
            self.to_world(self.bounds.min),
            self.to_world(self.bounds.max + vec2(1, 1)),
        )
    }

    pub fn from_world_unbound(&self, pos: vec2<FCoord>) -> vec2<ICoord> {
        (pos / self.cell_size).map(|x| x.floor().as_f32() as ICoord)
    }

    pub fn is_in_bounds(&self, pos: vec2<ICoord>) -> bool {
        let bounds = &self.bounds;
        bounds.min.x <= pos.x
            && pos.x <= bounds.max.x
            && bounds.min.y <= pos.y
            && pos.y <= bounds.max.y
    }

    pub fn from_world(&self, pos: vec2<FCoord>) -> Option<vec2<ICoord>> {
        let pos = self.from_world_unbound(pos);
        self.is_in_bounds(pos).then_some(pos)
    }

    pub fn tile_bounds(&self, pos: vec2<ICoord>) -> Aabb2<FCoord> {
        let pos = self.to_world(pos);
        Aabb2::point(pos).extend_positive(self.cell_size)
    }
}

pub struct Player {
    pub pos: vec2<ICoord>,
}

pub struct PlayerTrail {
    pub pos: vec2<ICoord>,
    /// Trail's lifetime in game turns.
    pub time_left: Turns,
}
