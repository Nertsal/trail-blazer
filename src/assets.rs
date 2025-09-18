use std::path::PathBuf;

use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub shaders: Shaders,
    pub sounds: Sounds,
    #[load(path = "dungeon-mode.ttf")]
    pub font: Rc<geng::Font>,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub crt: Rc<ugli::Program>,
}

#[derive(geng::asset::Load)]
pub struct Sounds {
    pub click: Rc<geng::Sound>,
    pub hover: Rc<geng::Sound>,
    pub walk: Rc<geng::Sound>,
    pub gameover: Rc<geng::Sound>,
    pub gather: Rc<geng::Sound>,
    pub score: Rc<geng::Sound>,
    pub stunned: Rc<geng::Sound>,
    pub teleport: Rc<geng::Sound>,
    pub throw_mushroom: Rc<geng::Sound>,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub background: PixelTexture,
    pub outline_corner_tl: PixelTexture,
    pub outline_corner_bl: PixelTexture,
    pub outline_corner_br: PixelTexture,
    pub outline_corner_tr: PixelTexture,
    pub outline_straight_up: PixelTexture,
    pub outline_straight_right: PixelTexture,
    #[load(list = "0..=1")]
    pub tiles: Vec<PixelTexture>,
    pub highlight_tile: PixelTexture,
    pub wall: PixelTexture,
    pub mushroom: PixelTexture,
    pub base: PixelTexture,
    pub characters: CharacterSprites,
    pub trail: TrailSprites,
    pub abilities: AbilitySprites,
    pub mushrooms_panel: PixelTexture,
    pub mushroom_slot: PixelTexture,
    pub mushroom_collected: PixelTexture,
    pub timer_frame: PixelTexture,
    pub score_panel: PixelTexture,
    pub menu_background: PixelTexture,
    pub join: PixelTexture,
    pub join_hover: PixelTexture,
    pub join_press: PixelTexture,
    pub character_panel: PixelTexture,
    pub name_panel: PixelTexture,
    pub button_prev: PixelTexture,
    pub button_prev_hover: PixelTexture,
    pub button_prev_press: PixelTexture,
    pub button_next: PixelTexture,
    pub button_next_hover: PixelTexture,
    pub button_next_press: PixelTexture,
}

#[derive(geng::asset::Load)]
pub struct CharacterSprites {
    pub ant: PixelTexture,
    pub bunny: PixelTexture,
    pub cat: PixelTexture,
    pub crab: PixelTexture,
    pub dinosaur: PixelTexture,
    pub dog: PixelTexture,
    pub elephant: PixelTexture,
    pub fishman: PixelTexture,
    pub fox: PixelTexture,
    pub frog: PixelTexture,
    pub ghost: PixelTexture,
    pub goat: PixelTexture,
    pub mouse: PixelTexture,
    pub panda: PixelTexture,
    pub penguin: PixelTexture,
    pub skeleton: PixelTexture,
    pub snake: PixelTexture,
    pub unicorn: PixelTexture,
}

#[derive(geng::asset::Load)]
pub struct AbilitySprites {
    pub sprint: PixelTexture,
    pub sprint_disable: PixelTexture,
    pub teleport: PixelTexture,
    pub teleport_disable: PixelTexture,
    pub throw: PixelTexture,
    pub throw_disable: PixelTexture,
}

#[derive(geng::asset::Load)]
pub struct TrailSprites {
    pub initial: PixelTexture,
    pub straight: PixelTexture,
    pub corner: PixelTexture,
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &()).await
    }
}

#[derive(Clone)]
pub struct PixelTexture {
    pub path: PathBuf,
    pub texture: Rc<ugli::Texture>,
}

impl Deref for PixelTexture {
    type Target = ugli::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Debug for PixelTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixelTexture")
            .field("path", &self.path)
            .field("texture", &"<texture data>")
            .finish()
    }
}

impl geng::asset::Load for PixelTexture {
    type Options = <ugli::Texture as geng::asset::Load>::Options;

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let path = path.to_owned();
        let texture = ugli::Texture::load(manager, &path, options);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Self {
                path,
                texture: Rc::new(texture),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
