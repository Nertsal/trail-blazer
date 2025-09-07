pub mod client;
pub mod shared;

use crate::interop::ClientId;

use geng::prelude::*;
use geng_utils::conversions::*;

pub type ICoord = i64;
pub type FCoord = R32;
pub type FTime = R32;
pub type Turns = i64;

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

    pub fn random_position(&self) -> vec2<ICoord> {
        let mut rng = thread_rng();
        vec2(
            rng.gen_range(self.bounds.min.x..=self.bounds.max.x),
            rng.gen_range(self.bounds.min.y..=self.bounds.max.y),
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Character {
    Bunny,
    Fox,
}

impl Character {
    pub fn random() -> Self {
        *[Self::Bunny, Self::Fox].choose(&mut thread_rng()).unwrap()
    }

    pub fn color(&self) -> Rgba<f32> {
        match self {
            Character::Bunny => Rgba::try_from("#5590B4").unwrap(),
            Character::Fox => Rgba::try_from("#B03B59").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTrail {
    pub player: ClientId,
    pub pos: vec2<ICoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: ClientId,
    pub character: Character,
    pub pos: vec2<ICoord>,
    pub speed: usize,
    pub submitted_move: Vec<vec2<ICoord>>,
    pub mushrooms: usize,
    pub stunned_duration: Option<Turns>,
    pub resolution_speed_left: usize,
}

impl Player {
    pub fn new(id: ClientId, character: Character, pos: vec2<ICoord>) -> Self {
        Self {
            id,
            character,
            pos,
            speed: 5,
            submitted_move: vec![],
            mushrooms: 0,
            stunned_duration: None,
            resolution_speed_left: 0,
        }
    }
}
