use crate::{
    assets::*,
    game::GameUi,
    model::{client::ClientModel, particles::ParticleKind, shared::Phase, *},
};

use geng::prelude::{itertools::Itertools, *};
use geng_utils::conversions::*;

const TARGET_SCREEN_SIZE: vec2<usize> = vec2(480, 320);

pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    pixel_texture: ugli::Texture,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            pixel_texture: geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1)),
        }
    }

    pub fn draw_game(&mut self, model: &mut ClientModel, framebuffer: &mut ugli::Framebuffer) {
        let pixel_scale = framebuffer.size().as_f32() / TARGET_SCREEN_SIZE.as_f32();
        let pixel_scale = pixel_scale.x.min(pixel_scale.y).floor().max(0.25);
        let size = (framebuffer.size().as_f32() / pixel_scale).map(|x| x.floor() as usize);
        geng_utils::texture::update_texture_size(&mut self.pixel_texture, size, self.geng.ugli());

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
            let player_pos = map.tile_bounds(player.pos).as_f32();
            geng_utils::texture::DrawTexture::new(texture)
                .fit(player_pos, vec2(0.5, 0.5))
                .colored(color)
                .draw(&model.camera, &self.geng, framebuffer);

            // Mushrooms
            let icon_size = map.cell_size.as_f32() * 0.15;
            let spacing = icon_size.x * 0.5;
            let n = player.mushrooms;
            let total_width = icon_size.x * n as f32 + spacing * n.saturating_sub(1) as f32;
            for i in 0..n {
                let pos = Aabb2::point(vec2(
                    player_pos.center().x + total_width * 0.5 * (i as f32 - (n as f32 - 1.0) / 2.0),
                    player_pos.max.y,
                ))
                .extend_symmetric(icon_size / 2.0);
                self.geng.draw2d().quad(
                    framebuffer,
                    &model.camera,
                    pos,
                    Rgba::try_from("#E5BD85").unwrap(),
                );
            }

            // State
            let state = if player.stunned_duration.is_some() {
                "stunned"
            } else if player.is_channeling {
                "teleporting"
            } else {
                ""
            };
            if !state.is_empty() {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &model.camera,
                    &draw2d::Text::unit(
                        self.assets.font.clone(),
                        state,
                        Rgba::try_from("#F6F5DE").unwrap(),
                    )
                    .transform(
                        mat3::translate(vec2(player_pos.center().x, player_pos.min.y))
                            * mat3::scale_uniform(
                                model.shared.map.cell_size.y.as_f32() * 0.1 * 0.6,
                            ),
                    ),
                );
            }
        }

        // Planned move
        if let Some(player) = model.shared.players.get(&model.player_id) {
            // Path
            let path = match &player.submitted_move {
                PlayerMove::Normal { path, .. } => Some(path.clone()),
                &PlayerMove::Throw { direction } => Some(
                    (0..=shared::THROW_SPEED)
                        .map(|i| player.pos + direction * i as ICoord)
                        .collect(),
                ),
                _ => None,
            };
            if let Some(path) = path {
                let skip = match model.shared.phase {
                    Phase::Planning { .. } => 0,
                    Phase::Resolution { .. } => {
                        player.resolution_speed_max - player.resolution_speed_left
                    }
                };
                for (&from, &at, &to) in path.iter().skip(skip).tuple_windows() {
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
            }

            // Target ghost
            let pos = match &player.submitted_move {
                PlayerMove::Normal { path, .. } => path.last().copied(),
                PlayerMove::TeleportActivate { teleport_to } => Some(*teleport_to),
                &PlayerMove::Throw { direction }
                    if matches!(model.shared.phase, Phase::Planning { .. }) =>
                {
                    Some(player.pos + direction * shared::THROW_SPEED as ICoord)
                }
                _ => None,
            };
            if let Some(pos) = pos
                && player.pos != pos
            {
                let texture = match player.submitted_move {
                    PlayerMove::Throw { .. } => &sprites.mushroom,
                    _ => get_character_sprite(&sprites.characters, player.character),
                };
                let pos = map.tile_bounds(pos).as_f32();
                geng_utils::texture::DrawTexture::new(texture)
                    .fit(pos, vec2(0.5, 0.5))
                    .colored(Rgba::try_from("#393b42").unwrap())
                    .draw(&model.camera, &self.geng, framebuffer);
            }
        }

        self.draw_pixels(model, framebuffer);
    }

    fn draw_pixels(&mut self, model: &ClientModel, final_buffer: &mut ugli::Framebuffer) {
        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.pixel_texture, self.geng.ugli());
        ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), None, None);

        for particle in &model.particles {
            let t = particle.lifetime.get_ratio().as_f32().sqrt();
            let color = match particle.kind {
                ParticleKind::Mushroom => Rgba::try_from("#E5BD85").unwrap(),
                ParticleKind::Stun => Rgba::try_from("#6D767B").unwrap(),
            };
            self.geng.draw2d().circle(
                framebuffer,
                &model.camera,
                particle.position.as_f32(),
                particle.radius.as_f32() * t,
                color,
            );
        }
        // Floating Text
        // for (text, position, size, color, lifetime) in query!(
        //     self.model.floating_texts,
        //     (&text, &position, &size, &color, &lifetime)
        // ) {
        //     let t = lifetime.get_ratio().as_f32().sqrt();
        //     self.util.draw_text(
        //         text,
        //         position.as_f32(),
        //         &self.context.assets.fonts.revolver_game,
        //         TextRenderOptions::new(size.as_f32() * t).color(*color),
        //         &model.camera,
        //         framebuffer,
        //     );
        // }

        geng_utils::texture::DrawTexture::new(&self.pixel_texture)
            .fit_screen(vec2(0.5, 0.5), final_buffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, final_buffer);
    }

    pub fn draw_game_ui(
        &self,
        model: &ClientModel,
        ui: &GameUi,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if let Some(player) = model.shared.players.get(&model.player_id) {
            // Abilities
            let texture = if player.cooldown_sprint > 0 {
                &self.assets.sprites.abilities.sprint_disable
            } else {
                &self.assets.sprites.abilities.sprint
            };
            let feedback = if player.cooldown_sprint <= 0 && ui.ability_sprint.hovered
                || matches!(
                    player.submitted_move,
                    PlayerMove::Normal { sprint: true, .. }
                ) {
                ui.ability_sprint.position.width() * 0.1
            } else {
                0.0
            };
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                ui.ability_sprint.position.extend_uniform(feedback),
                texture,
                Rgba::WHITE,
            );

            let texture = if player.cooldown_teleport > 0 {
                &self.assets.sprites.abilities.teleport_disable
            } else {
                &self.assets.sprites.abilities.teleport
            };
            let feedback = if player.cooldown_teleport <= 0 && ui.ability_teleport.hovered
                || matches!(
                    player.submitted_move,
                    PlayerMove::TeleportChanneling | PlayerMove::TeleportActivate { .. }
                )
                || player.is_channeling
            {
                ui.ability_teleport.position.width() * 0.1
            } else {
                0.0
            };
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                ui.ability_teleport.position.extend_uniform(feedback),
                texture,
                Rgba::WHITE,
            );

            let texture = if player.mushrooms == 0 {
                &self.assets.sprites.abilities.throw_disable
            } else {
                &self.assets.sprites.abilities.throw
            };
            let feedback = if player.mushrooms > 0 && ui.ability_throw.hovered
                || matches!(player.submitted_move, PlayerMove::Throw { .. })
            {
                ui.ability_throw.position.width() * 0.1
            } else {
                0.0
            };
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                ui.ability_throw.position.extend_uniform(feedback),
                texture,
                Rgba::WHITE,
            );

            // Mushrooms
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                ui.mushrooms.position,
                &self.assets.sprites.mushrooms_panel,
                Rgba::WHITE,
            );
            let mushroom_size = ui.mushrooms.position.size() * vec2(0.2, 1.0);
            for i in 0..5 {
                let pos = Aabb2::point(
                    ui.mushrooms.position.bottom_left() + vec2(mushroom_size.x * i as f32, 0.0),
                )
                .extend_positive(mushroom_size);
                let pos = Aabb2::point(pos.center()).extend_symmetric(
                    vec2(
                        self.assets.sprites.mushroom_slot.size().as_f32().aspect(),
                        1.0,
                    ) * mushroom_size.y
                        * 0.7
                        / 2.0,
                );
                let texture = if player.mushrooms <= i {
                    &self.assets.sprites.mushroom_slot
                } else {
                    &self.assets.sprites.mushroom_collected
                };
                self.geng.draw2d().textured_quad(
                    framebuffer,
                    &geng::PixelPerfectCamera,
                    pos,
                    texture,
                    Rgba::WHITE,
                );
            }
        }

        let map_bounds = model.shared.map.world_bounds().as_f32();
        let top = vec2(
            map_bounds.center().x,
            map_bounds.max.y + model.shared.map.cell_size.y.as_f32() * 1.5,
        );

        // Turn count
        self.geng.draw2d().draw2d(
            framebuffer,
            &model.camera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                format!("/{}", model.shared.turns_max),
                Rgba::try_from("#474C80").unwrap(),
            )
            .align_bounding_box(vec2(1.0, 0.5))
            .transform(
                mat3::translate(top - vec2(model.shared.map.cell_size.x.as_f32() * 0.1, 0.0))
                    * mat3::scale_uniform(model.shared.map.cell_size.y.as_f32() * 0.25),
            ),
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            &model.camera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                format!("{}", model.shared.turn_current),
                Rgba::try_from("#B4A091").unwrap(),
            )
            .align_bounding_box(vec2(1.0, 0.5))
            .transform(
                mat3::translate(top - vec2(model.shared.map.cell_size.x.as_f32() * 2.1, 0.0))
                    * mat3::scale_uniform(model.shared.map.cell_size.y.as_f32() * 0.25),
            ),
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            &model.camera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                "Turn",
                Rgba::try_from("#474C80").unwrap(),
            )
            .align_bounding_box(vec2(1.0, 0.5))
            .transform(
                mat3::translate(top - vec2(model.shared.map.cell_size.x.as_f32() * 3.1, 0.0))
                    * mat3::scale_uniform(model.shared.map.cell_size.y.as_f32() * 0.25),
            ),
        );

        // Turn timer
        let t = match model.shared.phase {
            Phase::Planning { time_left } => time_left.as_f32().max(0.0) / shared::TIME_PER_PLAN,
            _ => 0.0,
        };
        let timer_size = vec2(
            self.assets.sprites.timer_frame.size().as_f32().aspect(),
            1.0,
        ) * model.shared.map.cell_size.y.as_f32()
            * 0.75;
        let timer = Aabb2::point(top + vec2(model.shared.map.cell_size.x.as_f32() * 0.1, 0.0))
            .extend_symmetric(vec2(0.0, timer_size.y) / 2.0)
            .extend_right(timer_size.x);
        let timer_fill = timer.extend_uniform(-timer_size.y * 0.15);
        let timer_fill = timer_fill.extend_right(-timer_fill.width() * (1.0 - t));
        self.geng.draw2d().quad(
            framebuffer,
            &model.camera,
            timer_fill,
            Rgba::try_from("#B03B59").unwrap(),
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            &model.camera,
            timer,
            &self.assets.sprites.timer_frame,
            Rgba::WHITE,
        );

        // Score panel
        let screen_size = framebuffer.size().as_f32();
        let score_panel = Aabb2::point(screen_size)
            .extend_down(screen_size.y)
            .extend_left(self.assets.sprites.score_panel.size().as_f32().aspect() * screen_size.y);
        self.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            score_panel,
            &self.assets.sprites.score_panel,
            Rgba::WHITE,
        );

        // Personal score
        if let Some(player) = model.shared.players.get(&model.player_id) {
            let score = vec2(
                score_panel.center().x,
                score_panel.max.y - score_panel.height() * 0.1,
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                &geng::PixelPerfectCamera,
                &draw2d::Text::unit(
                    self.assets.font.clone(),
                    format!("{}", player.score),
                    Rgba::try_from("#E5BD85").unwrap(),
                )
                .align_bounding_box(vec2(0.5, 0.5))
                .transform(
                    mat3::translate(score) * mat3::scale_uniform(score_panel.height() / 30.0 * 0.6),
                ),
            );
        }

        // Leaderboard
        let top = vec2(
            score_panel.center().x,
            score_panel.max.y - score_panel.height() * 0.2,
        );
        let score_height = score_panel.height() / 50.0;
        let character_height = score_panel.height() / 15.0;
        let spacing = score_panel.height() / 25.0;
        let total_height = score_height + character_height + spacing;
        for (i, player) in model
            .shared
            .players
            .values()
            .sorted_by_key(|player| player.id)
            .enumerate()
        {
            let top = top + vec2(0.0, -total_height * i as f32);
            let color = player.character.color();
            self.geng.draw2d().draw2d(
                framebuffer,
                &geng::PixelPerfectCamera,
                &draw2d::Text::unit(self.assets.font.clone(), format!("{}", player.score), color)
                    .align_bounding_box(vec2(0.5, 0.5))
                    .transform(mat3::translate(top) * mat3::scale_uniform(score_height * 0.6)),
            );
            geng_utils::texture::DrawTexture::new(get_character_sprite(
                &self.assets.sprites.characters,
                player.character,
            ))
            .fit_height(
                Aabb2::point(top - vec2(0.0, score_height)).extend_down(character_height),
                0.5,
            )
            .colored(color)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        }
    }
}

fn get_character_sprite(sprites: &CharacterSprites, character: Character) -> &PixelTexture {
    match character {
        Character::Ant => &sprites.ant,
        Character::Bunny => &sprites.bunny,
        Character::Cat => &sprites.cat,
        Character::Crab => &sprites.crab,
        Character::Dinosaur => &sprites.dinosaur,
        Character::Dog => &sprites.dog,
        Character::Elephant => &sprites.elephant,
        Character::Fishman => &sprites.fishman,
        Character::Fox => &sprites.fox,
        Character::Frog => &sprites.frog,
        Character::Ghost => &sprites.ghost,
        Character::Goat => &sprites.goat,
        Character::Mouse => &sprites.mouse,
        Character::Panda => &sprites.panda,
        Character::Penguin => &sprites.penguin,
        Character::Skeleton => &sprites.skeleton,
        Character::Snake => &sprites.snake,
        Character::Unicorn => &sprites.unicorn,
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
