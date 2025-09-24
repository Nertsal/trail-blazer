use crate::assets::Assets;

use geng::prelude::*;

/// Max distance that the cursor can travel for a click to register as a stationary one.
const MAX_CLICK_DISTANCE: f32 = 1.0;

#[derive(Clone)]
pub struct UiContext {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub font: Rc<geng::Font>,

    pub cursor: CursorContext,

    pub real_time: f32,
    pub delta_time: f32,
    pub screen: Aabb2<f32>,
}

impl UiContext {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            font: assets.font.clone(),

            cursor: CursorContext::new(),

            screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
            real_time: 0.0,
            delta_time: 0.1,
        }
    }

    /// Should be called before layout.
    /// Updates input values.
    // TODO: use window from context
    pub fn update(&mut self, delta_time: f32, touch: bool) {
        self.real_time += delta_time;
        self.delta_time = delta_time;
        let window = self.geng.window();
        self.cursor.update(
            touch || geng_utils::key::is_key_pressed(window, [geng::MouseButton::Left]),
            geng_utils::key::is_key_pressed(window, [geng::MouseButton::Right]),
        );
    }

    /// Should be called after the layout.
    /// Reset accumulators to prepare for the next frame.
    pub fn frame_end(&mut self) {
        self.cursor.scroll = 0.0
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MouseButtonContext {
    /// Is the cursor currently pressed.
    pub down: bool,
    /// Was the cursor pressed last frame.
    pub was_down: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorContext {
    /// Position set outside of the update, synchronized in the update.
    next_position: vec2<f32>,
    pub position: vec2<f32>,
    /// Cursor position last frame.
    pub last_position: vec2<f32>,
    pub left: MouseButtonContext,
    pub right: MouseButtonContext,
    pub scroll: f32,
}

impl Default for CursorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorContext {
    pub fn new() -> Self {
        Self {
            next_position: vec2::ZERO,
            position: vec2::ZERO,
            last_position: vec2::ZERO,
            left: MouseButtonContext::default(),
            right: MouseButtonContext::default(),
            scroll: 0.0,
        }
    }

    pub fn scroll_dir(&self) -> i64 {
        if self.scroll == 0.0 {
            0
        } else {
            self.scroll.signum() as i64
        }
    }

    pub fn cursor_move(&mut self, pos: vec2<f32>) {
        self.next_position = pos;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Returns the delta the cursor has travelled since last frame.
    pub fn delta(&self) -> vec2<f32> {
        self.position - self.last_position
    }

    pub fn update(&mut self, is_down: bool, right_down: bool) {
        self.last_position = self.position;
        self.position = self.next_position;
        self.left.update(is_down);
        self.right.update(right_down);
    }
}

impl MouseButtonContext {
    pub fn update(&mut self, is_down: bool) {
        self.was_down = self.down;
        self.down = is_down;
    }
}

#[derive(Debug, Clone)]
pub struct WidgetState {
    // pub id: WidgetId,
    pub position: Aabb2<f32>,
    /// Whether to show the widget.
    pub visible: bool,
    pub hovered: bool,
    pub mouse_left: WidgetMouseState,
    pub mouse_right: WidgetMouseState,
    pub sfx_config: WidgetSfxConfig,
}

#[derive(Default, Debug, Clone)]
pub struct WidgetMouseState {
    /// Whether user has pressed on the widget since last frame.
    pub just_pressed: bool,
    /// Whether user is holding the mouse button down on the widget.
    pub pressed: Option<WidgetPressState>,
    /// Whether user has released the mouse button since last frame.
    pub just_released: bool,
    /// Set to `true` on frames when a press+release input was registered as a click.
    pub clicked: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetPressState {
    /// Cursor position where the press started.
    pub press_position: vec2<f32>,
    /// The duration of the press.
    pub duration: f32,
}

#[derive(Default, Debug, Clone)]
pub struct WidgetSfxConfig {
    pub hover: bool,
    pub left_click: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sfx(self, sfx_config: WidgetSfxConfig) -> Self {
        Self { sfx_config, ..self }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.position = position;
        if self.visible {
            let was_hovered = self.hovered;
            self.hovered = self.position.contains(context.cursor.position);

            self.mouse_left
                .update(context, self.hovered, &context.cursor.left);
            self.mouse_right
                .update(context, self.hovered, &context.cursor.right);

            if self.mouse_left.clicked && self.sfx_config.left_click {
                let mut sfx = context.assets.sounds.click.play();
                sfx.set_volume(0.5);
            }
            if !was_hovered && self.hovered && self.sfx_config.hover {
                let mut sfx = context.assets.sounds.hover.play();
                sfx.set_volume(0.5);
            }
        } else {
            self.mouse_left.just_released = self.mouse_left.pressed.is_some();
            self.mouse_right.just_released = self.mouse_right.pressed.is_some();

            self.hovered = false;
            self.mouse_left = WidgetMouseState::default();
            self.mouse_right = WidgetMouseState::default();
        }
    }

    // pub fn show(&mut self) {
    //     self.visible = true;
    // }

    // pub fn hide(&mut self) {
    //     self.visible = false;
    //     self.hovered = false;
    //     self.mouse_left = WidgetMouseState::default();
    //     self.mouse_right = WidgetMouseState::default();
    // }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            // id: WidgetId::default(),
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            mouse_left: WidgetMouseState::default(),
            mouse_right: WidgetMouseState::default(),
            sfx_config: WidgetSfxConfig::default(),
        }
    }
}

impl WidgetMouseState {
    pub fn update(&mut self, context: &UiContext, hovered: bool, mouse: &MouseButtonContext) {
        let was_pressed = self.pressed.is_some();
        // TODO: check for mouse being pressed and then dragged onto the widget
        let pressed = mouse.down && (was_pressed || hovered && !mouse.was_down);
        self.just_pressed = !was_pressed && pressed;
        self.just_released = was_pressed && !pressed;
        self.clicked = self.just_released
            && self.pressed.is_some_and(|state| {
                (state.press_position - context.cursor.position).len() < MAX_CLICK_DISTANCE
            });
        self.pressed = if pressed {
            match self.pressed {
                Some(mut state) => {
                    state.duration += context.delta_time;
                    Some(state)
                }
                None => Some(WidgetPressState {
                    press_position: context.cursor.position,
                    duration: 0.0,
                }),
            }
        } else {
            None
        };
    }
}

impl WidgetSfxConfig {
    pub fn hover() -> Self {
        Self {
            hover: true,
            ..default()
        }
    }

    pub fn hover_left() -> Self {
        Self {
            hover: true,
            left_click: true,
            ..default()
        }
    }
}
