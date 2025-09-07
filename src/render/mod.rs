use crate::{
    assets::*,
    model::{client::ClientModel, shared::Phase, *},
};

use geng::prelude::{itertools::Itertools, *};
use geng_utils::conversions::*;

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_game(&self, model: &mut ClientModel, framebuffer: &mut ugli::Framebuffer) {
        let sprites = &self.assets.sprites;

        // Background
        let background = &sprites.background;
        // geng_utils::tiled::tile_area();
        geng_utils::texture::DrawTexture::new(background)
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        let map = &model.shared.map;
        self.geng.draw2d().quad(
            framebuffer,
            &model.camera,
            map.world_bounds().extend_symmetric(map.cell_size).as_f32(),
            Rgba::try_from("#1A151F").unwrap(),
        );
        {
            // Map outline
            // corners
            let bl = map.tile_bounds(map.bounds.min + vec2(-1, -1)).as_f32();
            let br = map
                .tile_bounds(vec2(map.bounds.max.x, map.bounds.min.y) + vec2(1, -1))
                .as_f32();
            let tl = map
                .tile_bounds(vec2(map.bounds.min.x, map.bounds.max.y) + vec2(-1, 1))
                .as_f32();
            let tr = map.tile_bounds(map.bounds.max + vec2(1, 1)).as_f32();
            for (corner, texture) in [tl, bl, br, tr].into_iter().zip([
                &sprites.outline_corner_tl,
                &sprites.outline_corner_bl,
                &sprites.outline_corner_br,
                &sprites.outline_corner_tr,
            ]) {
                geng_utils::texture::DrawTexture::new(texture)
                    .fit(corner, vec2(0.5, 0.5))
                    .draw(&model.camera, &self.geng, framebuffer);
            }
            // straight - left
            let mut draw = geng_utils::texture::DrawTexture::new(&sprites.outline_straight_up);
            draw.target = Aabb2::from_corners(bl.top_left(), tl.bottom_right());
            draw.draw(&model.camera, &self.geng, framebuffer);
            // straight - right
            let mut draw = geng_utils::texture::DrawTexture::new(&sprites.outline_straight_up);
            draw.target = Aabb2::from_corners(br.top_left(), tr.bottom_right());
            draw.transformed(mat3::scale(vec2(-1.0, 1.0))).draw(
                &model.camera,
                &self.geng,
                framebuffer,
            );
            // straight - top
            let mut draw = geng_utils::texture::DrawTexture::new(&sprites.outline_straight_right);
            draw.target = Aabb2::from_corners(tl.bottom_right(), tr.top_left());
            draw.draw(&model.camera, &self.geng, framebuffer);
            // straight - bottom
            let mut draw = geng_utils::texture::DrawTexture::new(&sprites.outline_straight_right);
            draw.target = Aabb2::from_corners(bl.bottom_right(), br.top_left());
            draw.draw(&model.camera, &self.geng, framebuffer);
        }
        // Map tiles
        let mut rng = thread_rng();
        for x in map.bounds.min.x..=map.bounds.max.x {
            for y in map.bounds.min.y..=map.bounds.max.y {
                let pos = vec2(x, y);
                let tile = if map.walls.contains(&pos) {
                    &sprites.wall
                } else {
                    let variant = *model
                        .tile_variants
                        .entry(pos)
                        .or_insert_with(|| rng.gen_range(0..sprites.tiles.len()));
                    sprites
                        .tiles
                        .get(variant)
                        .unwrap_or(sprites.tiles.first().unwrap())
                };
                let pos = map.tile_bounds(pos).as_f32();
                geng_utils::texture::DrawTexture::new(tile)
                    .fit(pos, vec2(0.5, 0.5))
                    .draw(&model.camera, &self.geng, framebuffer);
            }
        }

        // Base
        let pos = map.tile_bounds(model.shared.base).as_f32();
        geng_utils::texture::DrawTexture::new(&self.assets.sprites.base)
            .fit(pos, vec2(0.5, 0.5))
            .draw(&model.camera, &self.geng, framebuffer);

        // Mushrooms
        for mushroom in &model.shared.mushrooms {
            let pos = map.tile_bounds(mushroom.position).as_f32();
            geng_utils::texture::DrawTexture::new(&self.assets.sprites.mushroom)
                .fit(pos, vec2(0.5, 0.5))
                .draw(&model.camera, &self.geng, framebuffer);
        }

        // Trails
        for trail in &model.shared.trails {
            let color = model
                .shared
                .players
                .get(&trail.player)
                .map(|player| player.character.color())
                .unwrap_or(Rgba::MAGENTA);

            let (texture, rotation, flip) = get_trail_render(&sprites.trail, trail);

            let pos = map.tile_bounds(trail.pos).as_f32();
            geng_utils::texture::DrawTexture::new(texture)
                .fit(pos, vec2(0.5, 0.5))
                .transformed(
                    mat3::rotate(rotation) * mat3::scale(vec2(1.0, if flip { -1.0 } else { 1.0 })),
                )
                .colored(color)
                .draw(&model.camera, &self.geng, framebuffer);
        }

        // Players
        for player in model.shared.players.values() {
            let color = player.character.color();
            let texture = get_character_sprite(&sprites.characters, player.character);
            let pos = map.tile_bounds(player.pos).as_f32();
            geng_utils::texture::DrawTexture::new(texture)
                .fit(pos, vec2(0.5, 0.5))
                .colored(color)
                .draw(&model.camera, &self.geng, framebuffer);
        }

        // Planned move
        if let Some(player) = model.shared.players.get(&model.player_id) {
            let skip = match model.shared.phase {
                Phase::Planning { .. } => 0,
                Phase::Resolution { .. } => {
                    player.resolution_speed_max - player.resolution_speed_left
                }
            };
            for (&from, &at, &to) in player.submitted_move.iter().skip(skip).tuple_windows() {
                let trail = &PlayerTrail {
                    player: player.id,
                    pos: at,
                    connection_from: Some(from),
                    connection_to: to,
                };

                let color = Rgba::try_from("#393b42").unwrap();

                let (texture, rotation, flip) = get_trail_render(&sprites.trail, trail);

                let pos = map.tile_bounds(at).as_f32();
                geng_utils::texture::DrawTexture::new(texture)
                    .fit(pos, vec2(0.5, 0.5))
                    .transformed(
                        mat3::rotate(rotation)
                            * mat3::scale(vec2(1.0, if flip { -1.0 } else { 1.0 })),
                    )
                    .colored(color)
                    .draw(&model.camera, &self.geng, framebuffer);
            }
            if let Some(&pos) = player.submitted_move.last()
                && player.pos != pos
            {
                let texture = get_character_sprite(&sprites.characters, player.character);
                let pos = map.tile_bounds(pos).as_f32();
                geng_utils::texture::DrawTexture::new(texture)
                    .fit(pos, vec2(0.5, 0.5))
                    .colored(Rgba::try_from("#393b42").unwrap())
                    .draw(&model.camera, &self.geng, framebuffer);
            }
        }

        self.draw_game_ui(model, framebuffer);
    }

    pub fn draw_game_ui(&self, model: &ClientModel, framebuffer: &mut ugli::Framebuffer) {
        if let Phase::Planning { time_left } = model.shared.phase {
            // Planning timer
            let map_bounds = model.shared.map.world_bounds().as_f32();
            self.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    format!("Planning: {:.1}s", time_left.as_f32().max(0.0)),
                    Rgba::try_from("#E5BD85").unwrap(),
                )
                .transform(
                    mat3::translate(vec2(
                        map_bounds.center().x,
                        map_bounds.max.y + model.shared.map.cell_size.y.as_f32() * 1.5,
                    )) * mat3::scale_uniform(0.25),
                ),
            );
        }
    }
}

fn get_character_sprite(sprites: &CharacterSprites, character: Character) -> &PixelTexture {
    match character {
        Character::Bunny => &sprites.bunny,
        Character::Fox => &sprites.fox,
    }
}

fn get_trail_render<'a>(
    sprites: &'a TrailSprites,
    trail: &PlayerTrail,
) -> (&'a PixelTexture, Angle<f32>, bool) {
    match trail.connection_from {
        None => (
            &sprites.initial,
            (trail.connection_to - trail.pos).as_f32().arg(),
            false,
        ),
        Some(from) => {
            if from.x == trail.connection_to.x {
                (&sprites.straight, Angle::from_degrees(90.0), false)
            } else if from.y == trail.connection_to.y {
                (&sprites.straight, Angle::ZERO, false)
            } else {
                let from_angle = (trail.pos - from).as_f32().arg();
                let to_angle = (trail.connection_to - trail.pos).as_f32().arg();
                let flip = from_angle.angle_to(to_angle) < Angle::ZERO;
                (&sprites.corner, from_angle, flip)
            }
        }
    }
}
