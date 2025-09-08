use crate::{
    assets::Assets,
    model::Character,
    ui::{UiContext, WidgetState},
};

use geng::prelude::*;
use geng_utils::conversions::*;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    connect: Option<String>,
    transition: Option<geng::state::Transition>,
    ui_context: UiContext,
    ui: MainMenuUi,
    framebuffer_size: vec2<usize>,

    characters: Vec<Character>,
    character_i: usize,
    colors: Vec<Rgba<f32>>,
    color_i: usize,
    name: String,
}

pub struct MainMenuUi {
    pub join: WidgetState,
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
            ui_context: UiContext::new(geng, assets),
            ui: MainMenuUi::new(geng, assets),
            framebuffer_size: vec2(1, 1),

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
        // let connection = geng::net::client::connect(&args.connect.unwrap())
        //     .await
        //     .unwrap();
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        self.ui_context.update(delta_time as f32);
        self.ui.update(&mut self.ui_context, self.framebuffer_size);

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
                async move {
                    let connection = geng::net::client::connect(&connect.unwrap()).await.unwrap();
                    crate::game::Game::new(&geng, &assets, connection).await
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
            geng::Event::KeyPress { key } => {
                if let geng::Key::Backspace = key {
                    self.name.pop();
                } else if let Some(char) = key_to_char(key)
                    && self.name.len() < 10
                {
                    self.name.push(char);
                }
            }
            _ => (),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#1A151F").unwrap()),
            None,
            None,
        );

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.menu_background)
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.character_panel)
            .fit(self.ui.character.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        geng_utils::texture::DrawTexture::new(crate::render::get_character_sprite(
            &self.assets.sprites.characters,
            self.characters[self.character_i],
        ))
        .fit(self.ui.character.position, vec2(0.5, 0.5))
        .colored(self.colors[self.color_i])
        .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.name_panel)
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

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.button_prev)
            .fit(self.ui.skin_prev.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        geng_utils::texture::DrawTexture::new(&self.assets.sprites.button_next)
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

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.button_prev)
            .fit(self.ui.color_prev.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
        geng_utils::texture::DrawTexture::new(&self.assets.sprites.button_next)
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

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.join)
            .fit(self.ui.join.position, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        self.ui_context.frame_end();
    }
}

impl MainMenuUi {
    pub fn new(_geng: &Geng, _assets: &Rc<Assets>) -> Self {
        Self {
            join: WidgetState::new(),
            name: WidgetState::new(),
            character: WidgetState::new(),
            skin_prev: WidgetState::new(),
            skin_text: WidgetState::new(),
            skin_next: WidgetState::new(),
            color_prev: WidgetState::new(),
            color_text: WidgetState::new(),
            color_next: WidgetState::new(),
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
