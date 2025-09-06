use crate::{assets::*, interop::*};

use geng::prelude::*;

pub struct Game {}

impl Game {
    pub async fn new(geng: &Geng, assets: &Rc<Assets>, connection: ClientConnection) -> Self {
        Self {}
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        todo!()
    }
}
