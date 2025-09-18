use geng_utils::bounded::Bounded;

use super::*;

use crate::{
    interop::{ClientId, ClientMessage, ServerMessage},
    model::{particles::*, shared::GameEvent},
};

pub struct ClientModel {
    pub player_id: ClientId,
    pub messages: Vec<ClientMessage>,
    pub camera: Camera2d,
    pub shared: shared::SharedModel,
    pub tile_variants: HashMap<vec2<ICoord>, usize>,
    pub spawn_particles: Vec<SpawnParticles>,
    pub particles: Vec<Particle>,
    pub floating_texts: Vec<FloatingText>,
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

            spawn_particles: Vec::new(),
            particles: Vec::new(),
            floating_texts: Vec::new(),

            tile_variants: HashMap::new(),
            shared: model,
        }
    }

    pub fn update(&mut self, delta_time: FTime) -> Vec<GameEvent> {
        let events = self.shared.update(delta_time);

        for event in &events {
            self.process_event(event);
        }

        for text in &mut self.floating_texts {
            text.lifetime.change(-delta_time);
            text.position += text.velocity * delta_time;
        }
        self.floating_texts
            .retain(|text| text.lifetime.is_above_min());

        for particle in &mut self.particles {
            particle.lifetime.change(-delta_time);
            particle.position += particle.velocity * delta_time;
        }
        self.particles
            .retain(|particle| particle.lifetime.is_above_min());

        self.particles.extend(
            std::mem::take(&mut self.spawn_particles)
                .into_iter()
                .flat_map(particles::spawn_particles),
        );

        events
    }

    pub fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => {
                self.messages.push(ClientMessage::Pong);
            }
            ServerMessage::Setup(_setup) => {}
            ServerMessage::StartResolution(model) => self.shared = model,
            ServerMessage::FinishResolution(model) => self.shared = model,
            ServerMessage::PlayerCustomization(player, customization) => {
                if let Some(player) = self.shared.players.get_mut(&player) {
                    player.customization = customization;
                }
            }
        }
    }

    fn process_event(&mut self, event: &GameEvent) {
        match *event {
            GameEvent::MushroomPickup(pos) => self.spawn_particles.push(SpawnParticles {
                kind: ParticleKind::Mushroom,
                distribution: ParticleDistribution::Circle {
                    center: self.shared.map.to_world_center(pos),
                    radius: r32(0.6),
                },
                ..default()
            }),
            GameEvent::MushroomsCollected(pos, n) => self.spawn_particles.push(SpawnParticles {
                kind: ParticleKind::Mushroom,
                density: r32(3.0 + 1.5 * n as f32),
                distribution: ParticleDistribution::Circle {
                    center: self.shared.map.to_world_center(pos),
                    radius: r32(0.6),
                },
                ..default()
            }),
            GameEvent::Score(score, pos) => self.floating_texts.push(FloatingText {
                text: format!("+{score}").into(),
                position: self.shared.map.to_world_center(pos),
                velocity: vec2(0.1, 0.5).as_r32(),
                size: r32(0.5),
                color: Rgba::try_from("#E5BD85").unwrap(),
                lifetime: Bounded::new_max(r32(1.0)),
            }),
            GameEvent::PlayerStunned(_, pos) => self.spawn_particles.push(SpawnParticles {
                kind: ParticleKind::Stun,
                density: r32(5.0),
                distribution: ParticleDistribution::Circle {
                    center: self.shared.map.to_world_center(pos),
                    radius: r32(0.6),
                },
                ..default()
            }),
            _ => {}
        }
    }
}
