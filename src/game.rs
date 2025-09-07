use crate::{
    assets::*,
    interop::*,
    model::{shared::Phase, *},
    render::GameRender,
    ui::{UiContext, WidgetState},
};

use geng::prelude::*;
use geng_utils::conversions::*;

pub struct Game {
    connection: ClientConnection,
    geng: Geng,
    assets: Rc<Assets>,
    ui_context: UiContext,
    render: GameRender,
    model: client::ClientModel,
    ui: GameUi,

    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<FCoord>,
    cursor_grid_pos: vec2<ICoord>,
    drag: Option<Drag>,
}

pub struct GameUi {
    pub ability_sprint: WidgetState,
    pub ability_teleport: WidgetState,
    pub ability_throw: WidgetState,
    pub mushrooms: WidgetState,
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
            ui_context: UiContext::new(geng, assets),
            render: GameRender::new(geng, assets),
            model: client::ClientModel::new(setup.player_id, setup.model),
            ui: GameUi::new(geng, assets),

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
            geng::Event::KeyPress { key } => match key {
                geng::Key::Digit1 => todo!(),
                geng::Key::Digit2 => todo!(),
                geng::Key::Digit3 => todo!(),
                _ => {}
            },
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
                    } else if path.len() <= player.speed()
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
        self.ui_context.update(delta_time as f32);
        self.ui.update(
            &mut self.ui_context,
            delta_time as f32,
            self.framebuffer_size,
        );
        self.ui_context.frame_end();

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

        self.render.draw_game(&mut self.model, framebuffer);
        self.render.draw_game_ui(&self.model, &self.ui, framebuffer);
    }
}

impl GameUi {
    pub fn new(_geng: &Geng, _assets: &Rc<Assets>) -> Self {
        Self {
            ability_sprint: WidgetState::new(),
            ability_teleport: WidgetState::new(),
            ability_throw: WidgetState::new(),
            mushrooms: WidgetState::new(),
        }
    }

    pub fn update(
        &mut self,
        context: &mut UiContext,
        delta_time: f32,
        framebuffer_size: vec2<usize>,
    ) {
        let screen = Aabb2::ZERO.extend_positive(framebuffer_size.as_f32());
        context.screen = screen;
        context.update(delta_time);

        let layout_size = screen.height() * 0.05;

        let ability_size = vec2::splat(2.5 * layout_size);
        let mut pos = screen.bottom_left() + vec2::splat(1.0 * layout_size);

        for ability in [
            &mut self.ability_sprint,
            &mut self.ability_teleport,
            &mut self.ability_throw,
        ] {
            ability.update(Aabb2::point(pos).extend_positive(ability_size), context);
            pos.x += ability_size.x;
        }

        self.mushrooms.update(
            Aabb2::point(pos).extend_positive(vec2(ability_size.x * 5.0, ability_size.y)),
            context,
        );
    }
}
