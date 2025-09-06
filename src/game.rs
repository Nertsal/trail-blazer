use crate::{assets::*, interop::*, model::*};

use geng::prelude::*;

pub struct Game {
    connection: ClientConnection,
    sends: std::sync::mpsc::Receiver<ClientMessage>,
    geng: Geng,
    assets: Rc<Assets>,
    model: Model,
}

impl Game {
    pub async fn new(geng: &Geng, assets: &Rc<Assets>, mut connection: ClientConnection) -> Self {
        let ServerMessage::Setup(setup) = connection.next().await.unwrap().unwrap() else {
            unreachable!()
        };
        let (sender, sends) = std::sync::mpsc::channel();

        Self {
            connection,
            sends,
            geng: geng.clone(),
            assets: assets.clone(),
            model: Model::new(setup.map_size),
        }
    }

    fn handle_event(&mut self, event: geng::Event) {}
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn update(&mut self, delta_time: f64) {
        for message in self.connection.new_messages() {
            let message = message.unwrap();
            self.model.send(message);
        }
        while let Ok(message) = self.sends.try_recv() {
            self.connection.send(message);
        }
        let delta_time = FTime::new(delta_time as f32);
        self.model.update(delta_time);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}
