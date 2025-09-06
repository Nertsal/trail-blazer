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
        let map = model.map.world_bounds();
        Self {
            player_id,
            messages: Vec::new(),
            camera: Camera2d {
                center: map.center().as_f32(),
                rotation: Angle::ZERO,
                fov: Camera2dFov::Cover {
                    width: map.width().as_f32(),
                    height: map.height().as_f32(),
                    scale: 1.0,
                },
            },

            shared: model,
        }
    }

    pub fn update(&mut self, delta_time: FTime) {}

    pub fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => {
                self.messages.push(ClientMessage::Pong);
            }
            ServerMessage::Setup(_setup) => {}
        }
    }
}
