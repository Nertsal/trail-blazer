use super::*;

pub struct ServerModel {
    pub shared: shared::SharedModel,
}

impl ServerModel {
    pub fn new(model: shared::SharedModel) -> Self {
        Self { shared: model }
    }
}
