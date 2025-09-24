use crate::{
    assets::Assets,
    model::Character,
    ui::{UiContext, WidgetSfxConfig, WidgetState},
};

use geng::prelude::*;
use geng_utils::conversions::*;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    connect: Option<String>,
    transition: Option<geng::state::Transition>,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    ui_context: UiContext,
    ui: MainMenuUi,
    framebuffer_size: vec2<usize>,
    post_texture: ugli::Texture,
    time: f32,
    active_touch: Option<u64>,

    characters: Vec<Character>,
    character_i: usize,
    colors: Vec<Rgba<f32>>,
    color_i: usize,
    name: String,
}

pub struct MainMenuUi {
    pub join: WidgetState,
    pub spectate: WidgetState,
    pub name: WidgetState,
    pub character: WidgetState,
    pub skin_prev: WidgetState,
    pub skin_text: WidgetState,
    pub skin_next: WidgetState,
    pub color_prev: WidgetState,
    pub color_text: WidgetState,
    pub color_next: WidgetState,
}

impl MainMenu {
    pub async fn new(geng: &Geng, assets: &Rc<Assets>, connect: Option<String>) -> Self {
        let characters: Vec<Character> = Character::all().into();
        let character = Character::random();
        let colors = vec![
            Rgba::try_from("#6D767B").unwrap(),
            Rgba::try_from("#5590B4").unwrap(),
            Rgba::try_from("#C68B60").unwrap(),
            Rgba::try_from("#B158D3").unwrap(),
            Rgba::try_from("#474C80").unwrap(),
            Rgba::try_from("#B03B59").unwrap(),
            Rgba::try_from("#6BB453").unwrap(),
            Rgba::try_from("#69469B").unwrap(),
            Rgba::try_from("#3C7253").unwrap(),
            Rgba::try_from("#6A2F56").unwrap(),
        ];
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            connect,
            transition: None,
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            ui_context: UiContext::new(geng, assets),
            ui: MainMenuUi::new(geng, assets),
            framebuffer_size: vec2(1, 1),
            post_texture: geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1)),
            time: 0.0,
            active_touch: None,

            character_i: characters
                .iter()
                .position(|char| *char == character)
                .unwrap_or(0),
            characters,
            color_i: colors
                .iter()
                .position(|color| *color == character.color())
                .unwrap_or(0),
            colors,
            name: String::new(),
        }
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let transition = self.transition.take();
        if transition.is_some() {
            self.geng.window().stop_text_edit();
        }
        transition
    }

    fn update(&mut self, delta_time: f64) {
        self.time += delta_time as f32;
        self.ui_context
            .update(delta_time as f32, self.active_touch.is_some());
        self.ui.update(&mut self.ui_context, self.framebuffer_size);

        if self.ui.name.mouse_left.clicked {
            self.geng.window().start_text_edit(&self.name);
        }

        if self.ui.color_prev.mouse_left.clicked {
            self.color_i = if self.color_i == 0 {
                self.colors.len() - 1
            } else {
                self.color_i - 1
            };
        }
        if self.ui.color_next.mouse_left.clicked {
            self.color_i = if self.color_i == self.colors.len() - 1 {
                0
            } else {
                self.color_i + 1
            };
        }

        if self.ui.skin_prev.mouse_left.clicked {
            self.character_i = if self.character_i == 0 {
                self.characters.len() - 1
            } else {
                self.character_i - 1
            };
            let character = self.characters[self.character_i];
            self.color_i = self
                .colors
                .iter()
                .position(|color| *color == character.color())
                .unwrap_or(0);
        }
        if self.ui.skin_next.mouse_left.clicked {
            self.character_i = if self.character_i == self.characters.len() - 1 {
                0
            } else {
                self.character_i + 1
            };
            let character = self.characters[self.character_i];
            self.color_i = self
                .colors
                .iter()
                .position(|color| *color == character.color())
                .unwrap_or(0);
        }

        if self.ui.join.mouse_left.clicked {
            let future = {
                let geng = self.geng.clone();
                let assets = self.assets.clone();
                let connect = self.connect.clone();
                let customization = crate::model::PlayerCustomization {
                    name: self.name.clone(),
                    character: self.characters[self.character_i],
                    color: self.colors[self.color_i],
                };
                async move {
                    let connection = geng::net::client::connect(&connect.unwrap()).await.unwrap();
                    crate::game::Game::new(&geng, &assets, connection, customization).await
                }
            };
            let state = {
                geng::LoadingScreen::new(
                    &self.geng,
                    geng::EmptyLoadingScreen::new(&self.geng),
                    future,
                )
            };
            self.transition = Some(geng::state::Transition::Push(Box::new(state)));
        }
        if self.ui.spectate.mouse_left.clicked {
            let future = {
                let geng = self.geng.clone();
                let assets = self.assets.clone();
                let connect = self.connect.clone();
                async move {
                    let connection = geng::net::client::connect(&connect.unwrap()).await.unwrap();
                    crate::game::Game::new_spectator(&geng, &assets, connection).await
                }
            };
            let state = {
                geng::LoadingScreen::new(
                    &self.geng,
                    geng::EmptyLoadingScreen::new(&self.geng),
                    future,
                )
            };
            self.transition = Some(geng::state::Transition::Push(Box::new(state)));
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.ui_context.cursor.cursor_move(touch.position.as_f32());
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.ui_context.cursor.cursor_move(touch.position.as_f32());
            }
            geng::Event::TouchEnd(touch) if Some(touch.id) == self.active_touch => {
                self.ui_context.cursor.cursor_move(touch.position.as_f32());
            }
            // geng::Event::KeyPress { key } => {
            //     if let geng::Key::Backspace = key {
            //         self.name.pop();
            //     } else if let Some(char) = key_to_char(key)
            //         && self.name.len() < 10
            //     {
            //         self.name.push(char);
            //     }
            // }
            geng::Event::EditText(name) => self.name = name,
            _ => (),
        }
    }

    fn draw(&mut self, final_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = final_buffer.size();
        geng_utils::texture::update_texture_size(
            &mut self.post_texture,
            final_buffer.size(),
            self.geng.ugli(),
        );
        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.post_texture, self.geng.ugli());
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#1A151F").unwrap()),
            None,
            None,
        );

        let sprites = &self.assets.sprites;
        let button_variant = |state: &WidgetState, normal, hover, press| {
            if state.mouse_left.pressed.is_some() {
                press
            } else if state.hovered {
                hover
            } else {
                normal
            }
        };

        geng_utils::texture::DrawTexture::new(&sprites.menu_background)
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        geng_utils::texture::DrawTexture::new(&sprites.character_panel)
            .fit(self.ui.character.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        geng_utils::texture::DrawTexture::new(crate::render::get_character_sprite(
            &sprites.characters,
            self.characters[self.character_i],
        ))
        .fit(self.ui.character.position, vec2(0.5, 0.5))
        .colored(self.colors[self.color_i])
        .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        geng_utils::texture::DrawTexture::new(&sprites.name_panel)
            .fit(self.ui.name.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        self.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                if self.name.is_empty() {
                    "name.."
                } else {
                    &self.name
                },
                Rgba::try_from("#42343B").unwrap(),
            )
            .fit_into(
                self.ui
                    .name
                    .position
                    .extend_uniform(-4.0 / 12.0 * self.ui.name.position.height()),
            ),
        );

        let texture = button_variant(
            &self.ui.skin_prev,
            &sprites.button_prev,
            &sprites.button_prev_hover,
            &sprites.button_prev_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.skin_prev.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        let texture = button_variant(
            &self.ui.skin_next,
            &sprites.button_next,
            &sprites.button_next_hover,
            &sprites.button_next_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.skin_next.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        self.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                "skin",
                Rgba::try_from("#42343B").unwrap(),
            )
            .fit_into(self.ui.skin_text.position),
        );

        let texture = button_variant(
            &self.ui.color_prev,
            &sprites.button_prev,
            &sprites.button_prev_hover,
            &sprites.button_prev_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.color_prev.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        let texture = button_variant(
            &self.ui.color_next,
            &sprites.button_next,
            &sprites.button_next_hover,
            &sprites.button_next_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.color_next.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        self.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Text::unit(
                self.assets.font.clone(),
                "color",
                Rgba::try_from("#42343B").unwrap(),
            )
            .fit_into(self.ui.color_text.position),
        );

        let texture = button_variant(
            &self.ui.join,
            &sprites.join,
            &sprites.join_hover,
            &sprites.join_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.join.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        let texture = button_variant(
            &self.ui.spectate,
            &sprites.spectate,
            &sprites.spectate_hover,
            &sprites.spectate_press,
        );
        geng_utils::texture::DrawTexture::new(texture)
            .fit(self.ui.spectate.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        self.ui_context.frame_end();

        ugli::draw(
            final_buffer,
            &self.assets.shaders.crt,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            ugli::uniforms! {
                u_texture: &self.post_texture,
                u_curvature: 50.0,
                u_vignette_multiplier: 0.1,
                u_scanlines_multiplier: 0.1,
                u_time: self.time,
            },
            ugli::DrawParameters::default(),
        );
    }
}

impl MainMenuUi {
    pub fn new(_geng: &Geng, _assets: &Rc<Assets>) -> Self {
        Self {
            join: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            spectate: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            name: WidgetState::new(),
            character: WidgetState::new(),
            skin_prev: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            skin_text: WidgetState::new(),
            skin_next: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            color_prev: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            color_text: WidgetState::new(),
            color_next: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
        }
    }

    pub fn update(&mut self, context: &mut UiContext, framebuffer_size: vec2<usize>) {
        let screen = Aabb2::ZERO.extend_positive(framebuffer_size.as_f32());
        context.screen = screen;

        let width = screen.width().min(screen.height() * 16.0 / 9.0);
        let size = vec2(width, width * 9.0 / 16.0);
        let screen = Aabb2::point(screen.center()).extend_symmetric(size / 2.0);
        let main = Aabb2::from_corners(
            screen.top_left() + vec2(98.0, -91.0) / vec2(320.0, 180.0) * screen.size(),
            screen.top_left() + vec2(221.0, -152.0) / vec2(320.0, 180.0) * screen.size(),
        );

        let character = Aabb2::from_corners(
            main.top_left() + vec2(22.0, -7.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(45.0, -30.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.character.update(character, context);

        let name = Aabb2::from_corners(
            main.top_left() + vec2(13.0, -38.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(58.0, -49.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.name.update(name, context);

        let join = Aabb2::from_corners(
            main.top_left() + vec2(72.0, -37.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(108.0, -50.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.join.update(join, context);

        let spectate = Aabb2::from_corners(
            main.top_left() + vec2(130.0, -37.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(172.0, -50.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.spectate.update(spectate, context);

        let skin_prev = Aabb2::from_corners(
            main.top_left() + vec2(56.0, -10.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(59.0, -14.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.skin_prev.update(skin_prev, context);

        let skin_text = Aabb2::from_corners(
            main.top_left() + vec2(60.0, -9.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(102.0, -14.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.skin_text.update(skin_text, context);

        let skin_next = Aabb2::from_corners(
            main.top_left() + vec2(103.0, -10.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(106.0, -14.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.skin_next.update(skin_next, context);

        let color_prev = Aabb2::from_corners(
            main.top_left() + vec2(56.0, -21.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(59.0, -25.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.color_prev.update(color_prev, context);

        let color_text = Aabb2::from_corners(
            main.top_left() + vec2(60.0, -20.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(102.0, -25.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.color_text.update(color_text, context);

        let color_next = Aabb2::from_corners(
            main.top_left() + vec2(103.0, -21.0) / vec2(124.0, 62.0) * main.size(),
            main.top_left() + vec2(106.0, -25.0) / vec2(124.0, 62.0) * main.size(),
        );
        self.color_next.update(color_next, context);
    }
}

fn key_to_char(key: geng::Key) -> Option<char> {
    match key {
        geng::Key::A => Some('a'),
        geng::Key::B => Some('b'),
        geng::Key::C => Some('c'),
        geng::Key::D => Some('d'),
        geng::Key::E => Some('e'),
        geng::Key::F => Some('f'),
        geng::Key::G => Some('g'),
        geng::Key::H => Some('h'),
        geng::Key::I => Some('i'),
        geng::Key::J => Some('j'),
        geng::Key::K => Some('k'),
        geng::Key::L => Some('l'),
        geng::Key::M => Some('m'),
        geng::Key::N => Some('n'),
        geng::Key::O => Some('o'),
        geng::Key::P => Some('p'),
        geng::Key::Q => Some('q'),
        geng::Key::R => Some('r'),
        geng::Key::S => Some('s'),
        geng::Key::T => Some('t'),
        geng::Key::U => Some('u'),
        geng::Key::V => Some('v'),
        geng::Key::W => Some('w'),
        geng::Key::X => Some('x'),
        geng::Key::Y => Some('y'),
        geng::Key::Z => Some('z'),
        _ => None,
    }
}
