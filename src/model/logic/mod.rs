use super::*;

use crate::interop::ServerMessage;

impl Model {
    pub fn update(&mut self, delta_time: FTime) {}

    pub fn send(&mut self, message: ServerMessage) {}
}
