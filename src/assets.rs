use std::path::PathBuf;

use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    #[load(path = "dungeon-mode.ttf")]
    pub font: Rc<geng::Font>,
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
    pub tile: PixelTexture,
    pub wall: PixelTexture,
    pub char_bunny: PixelTexture,
    pub char_fox: PixelTexture,
    pub mushroom: PixelTexture,
    pub base: PixelTexture,
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
