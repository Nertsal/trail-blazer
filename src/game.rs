use crate::{assets::*, interop::*, model::*, render::GameRender};

use geng::prelude::*;

pub struct Game {
    connection: ClientConnection,
    geng: Geng,
    assets: Rc<Assets>,
    render: GameRender,
    model: client::ClientModel,
}

impl Game {
    pub async fn new(geng: &Geng, assets: &Rc<Assets>, mut connection: ClientConnection) -> Self {
        let ServerMessage::Setup(setup) = connection.next().await.unwrap().unwrap() else {
            unreachable!()
        };

        Self {
            connection,
            geng: geng.clone(),
            assets: assets.clone(),
            render: GameRender::new(geng, assets),
            model: client::ClientModel::new(setup.player_id, setup.model),
        }
    }

    fn handle_event(&mut self, event: geng::Event) {}
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn update(&mut self, delta_time: f64) {
        // Process server messages
        for message in self.connection.new_messages() {
            let message = message.unwrap();
            self.model.handle_message(message);
        }

        // Send client messages
        for message in std::mem::take(&mut self.model.messages) {
            self.connection.send(message);
        }

        // Update model
        let delta_time = FTime::new(delta_time as f32);
        self.model.update(delta_time);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#1A151F").unwrap()),
            None,
            None,
        );
        self.render.draw_game(&self.model, framebuffer);
    }
}
