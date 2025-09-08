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
                geng::Key::Digit1 => self.ability_sprint(),
                geng::Key::Digit2 => self.ability_teleport(),
                geng::Key::Digit3 => self.ability_throw(),
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
                match &mut player.submitted_move {
                    PlayerMove::Normal { path, .. } => {
                        *path = vec![player.pos];
                    }
                    _ => {
                        player.submitted_move = PlayerMove::Normal {
                            path: vec![player.pos],
                            sprint: false,
                        };
                        self.connection
                            .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
                    }
                }
            }
        } else if let Some(player) = self.model.shared.players.get_mut(&self.model.player_id) {
            if player.is_channeling {
                player.submitted_move = PlayerMove::TeleportActivate {
                    teleport_to: self.cursor_grid_pos,
                };
                self.connection
                    .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
            } else if let PlayerMove::Throw { direction } = &mut player.submitted_move {
                let new_dir = (self.cursor_grid_pos - player.pos).map(|x| x.clamp_abs(1));
                if new_dir.x == 0 || new_dir.y == 0 {
                    *direction = new_dir;
                    self.connection
                        .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
                }
            }
        }
    }

    fn mouse_release(&mut self) {
        if let Some(drag) = self.drag.take() {
            match drag.target {
                DragTarget::Player { .. } => {
                    let Some(player) = self.model.shared.players.get_mut(&self.model.player_id)
                    else {
                        return;
                    };
                    // player.submitted_move = path.clone();
                    self.connection
                        .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
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

                    let sprint = matches!(
                        player.submitted_move,
                        PlayerMove::Normal { sprint: true, .. }
                    );

                    let mut update = false;
                    if path
                        .len()
                        .checked_sub(2)
                        .and_then(|i| path.get(i))
                        .is_some_and(|&prev_pos| prev_pos == self.cursor_grid_pos)
                    {
                        // Cancel last move
                        path.pop();
                        update = true;
                    } else if path.len() <= player.speed(sprint)
                        && !path.contains(&self.cursor_grid_pos)
                        && let Some(&last) = path.last()
                        && shared::are_adjacent(last, self.cursor_grid_pos)
                        && !self.model.shared.map.walls.contains(&self.cursor_grid_pos)
                        && self.model.shared.map.is_in_bounds(self.cursor_grid_pos)
                    {
                        // Add tile
                        path.push(self.cursor_grid_pos);
                        update = true;
                    }

                    if update {
                        match &mut player.submitted_move {
                            PlayerMove::Normal {
                                path: move_path, ..
                            } => *move_path = path.clone(),
                            _ => {
                                player.submitted_move = PlayerMove::Normal {
                                    path: path.clone(),
                                    sprint: false,
                                };
                            }
                        }
                        self.connection
                            .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
                    }
                }
            }
        }
    }

    fn ability_sprint(&mut self) {
        let Some(player) = self.model.shared.players.get_mut(&self.model.player_id) else {
            return;
        };
        if player.cooldown_sprint > 0 {
            return;
        }
        match &mut player.submitted_move {
            PlayerMove::Normal { sprint, .. } => *sprint = !*sprint,
            _ => {
                player.submitted_move = PlayerMove::Normal {
                    path: vec![player.pos],
                    sprint: true,
                };
                self.connection
                    .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
            }
        }
    }

    fn ability_teleport(&mut self) {
        let Some(player) = self.model.shared.players.get_mut(&self.model.player_id) else {
            return;
        };
        if player.cooldown_teleport > 0 {
            return;
        }
        match player.submitted_move {
            PlayerMove::TeleportChanneling | PlayerMove::TeleportActivate { .. } => {
                player.submitted_move = PlayerMove::default()
            }
            _ => {
                if !player.is_channeling {
                    player.submitted_move = PlayerMove::TeleportChanneling;
                }
            }
        };
        self.connection
            .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
    }

    fn ability_throw(&mut self) {
        let Some(player) = self.model.shared.players.get_mut(&self.model.player_id) else {
            return;
        };
        player.submitted_move = match player.submitted_move {
            PlayerMove::Throw { .. } => PlayerMove::default(),
            _ => PlayerMove::Throw {
                direction: vec2(1, 0),
            },
        };
        self.connection
            .send(ClientMessage::SubmitMove(player.submitted_move.clone()));
    }
}

impl geng::State for Game {
    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn update(&mut self, delta_time: f64) {
        self.ui_context.cursor.cursor_move(self.cursor_pos.as_f32());
        self.ui_context.update(delta_time as f32);
        self.ui.update(&mut self.ui_context, self.framebuffer_size);

        if self.ui.ability_sprint.mouse_left.just_pressed {
            self.ability_sprint();
        }
        if self.ui.ability_teleport.mouse_left.clicked {
            self.ability_teleport();
        }
        if self.ui.ability_throw.mouse_left.clicked {
            self.ability_throw();
        }

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
        self.ui_context.frame_end();
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

    pub fn update(&mut self, context: &mut UiContext, framebuffer_size: vec2<usize>) {
        let screen = Aabb2::ZERO.extend_positive(framebuffer_size.as_f32());
        context.screen = screen;

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

        let mushrooms_size = vec2(
            context
                .assets
                .sprites
                .mushrooms_panel
                .size()
                .as_f32()
                .aspect(),
            1.0,
        ) * 2.5
            * 0.642857
            * layout_size;
        self.mushrooms.update(
            Aabb2::point(pos + vec2(0.0, ability_size.y / 2.0))
                .extend_right(mushrooms_size.x)
                .extend_symmetric(vec2(0.0, mushrooms_size.y / 2.0)),
            context,
        );
    }
}
