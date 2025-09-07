use super::*;

use crate::interop::{ClientId, ClientMessage, ServerMessage};

pub struct ClientModel {
    pub player_id: ClientId,
    pub messages: Vec<ClientMessage>,
    pub shared: shared::SharedModel,
    pub camera: Camera2d,
}

impl ClientModel {
    pub fn new(player_id: ClientId, model: shared::SharedModel) -> Self {
        let map = model.map.world_bounds().as_f32();
        Self {
            player_id,
            messages: Vec::new(),
            camera: Camera2d {
                center: map.center(),
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: map.width() * 1.8,
                    height: map.height() * 1.8,
                    scale: 1.0,
                },
            },

            shared: model,
        }
    }

    pub fn update(&mut self, delta_time: FTime) {
        self.shared.update(delta_time);
    }

    pub fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => {
                self.messages.push(ClientMessage::Pong);
            }
            ServerMessage::Setup(_setup) => {}
            ServerMessage::StartResolution(model) => self.shared = model,
            ServerMessage::FinishResolution(model) => self.shared = model,
        }
    }
}
