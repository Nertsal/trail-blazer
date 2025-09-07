use crate::{
    assets::*,
    interop::*,
    model::{shared::Phase, *},
    render::GameRender,
};

use geng::prelude::*;
use geng_utils::conversions::*;

pub struct Game {
    connection: ClientConnection,
    geng: Geng,
    assets: Rc<Assets>,
    render: GameRender,
    model: client::ClientModel,

    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<FCoord>,
    cursor_grid_pos: vec2<ICoord>,
    drag: Option<Drag>,
}

pub struct Drag {
    pub target: DragTarget,
}

pub enum DragTarget {
    Player { path: Vec<vec2<ICoord>> },
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

            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            cursor_grid_pos: vec2::ZERO,
            drag: None,
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => self.mouse_press(),
            geng::Event::MouseRelease {
                button: geng::MouseButton::Left,
            } => self.mouse_release(),
            geng::Event::CursorMove { position } => self.cursor_move(position),
            _ => {}
        }
    }

    fn mouse_press(&mut self) {
        if let Some(player) = self
            .model
            .shared
            .players
            .values_mut()
            .find(|player| player.pos == self.cursor_grid_pos)
        {
            // Drag player
            if player.id == self.model.player_id
                && let Phase::Planning { .. } = self.model.shared.phase
                && player.stunned_duration.is_none()
            {
                self.drag = Some(Drag {
                    target: DragTarget::Player {
                        path: vec![player.pos],
                    },
                });
                player.submitted_move = vec![player.pos];
            }
        }
    }

    fn mouse_release(&mut self) {
        if let Some(drag) = self.drag.take() {
            match drag.target {
                DragTarget::Player { path } => {
                    let Some(player) = self.model.shared.players.get_mut(&self.model.player_id)
                    else {
                        return;
                    };
                    player.submitted_move = path.clone();
                    self.connection.send(ClientMessage::SubmitMove(path));
                }
            }
        }
    }

    fn cursor_move(&mut self, position: vec2<f64>) {
        self.cursor_pos = position;
        self.cursor_world_pos = self
            .model
            .camera
            .screen_to_world(self.framebuffer_size.as_f32(), position.as_f32())
            .as_r32();
        self.cursor_grid_pos = self
            .model
            .shared
            .map
            .from_world_unbound(self.cursor_world_pos);

        if let Some(drag) = &mut self.drag {
            match &mut drag.target {
                DragTarget::Player { path } => {
                    let Some(player) = self.model.shared.players.get_mut(&self.model.player_id)
                    else {
                        return;
                    };
                    if path
                        .len()
                        .checked_sub(2)
                        .and_then(|i| path.get(i))
                        .is_some_and(|&prev_pos| prev_pos == self.cursor_grid_pos)
                    {
                        // Cancel last move
                        path.pop();
                        player.submitted_move = path.clone();
                        self.connection
                            .send(ClientMessage::SubmitMove(path.clone()));
                    } else if path.len() <= player.speed
                        && !path.contains(&self.cursor_grid_pos)
                        && let Some(&last) = path.last()
                        && shared::are_adjacent(last, self.cursor_grid_pos)
                        && !self.model.shared.map.walls.contains(&self.cursor_grid_pos)
                        && self.model.shared.map.is_in_bounds(self.cursor_grid_pos)
                    {
                        // Add tile
                        path.push(self.cursor_grid_pos);
                        player.submitted_move = path.clone();
                        self.connection
                            .send(ClientMessage::SubmitMove(path.clone()));
                    }
                }
            }
        }
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn update(&mut self, delta_time: f64) {
        // Process server messages
        for message in self.connection.new_messages() {
            let message = message.unwrap();
            if let ServerMessage::StartResolution(_) = message
                && let Some(drag) = &self.drag
                && let DragTarget::Player { .. } = &drag.target
            {
                self.drag = None;
            }
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
        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(Rgba::try_from("#1A151F").unwrap()),
            None,
            None,
        );
        self.render.draw_game(&self.model, framebuffer);
    }
}
