use super::*;

use geng_utils::bounded::Bounded;

#[derive(Debug, Clone)]
pub struct FloatingText {
    pub text: Rc<str>,
    pub position: vec2<FCoord>,
    pub velocity: vec2<FCoord>,
    pub size: FCoord,
    pub color: Rgba<f32>,
    pub lifetime: Bounded<FTime>,
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub kind: ParticleKind,
    pub position: vec2<FCoord>,
    pub radius: FCoord,
    pub size_function: SizeFunction,
    pub velocity: vec2<FCoord>,
    pub lifetime: Bounded<FTime>,
}

#[derive(Debug, Clone)]
pub struct SpawnParticles {
    pub kind: ParticleKind,
    pub density: R32,
    pub distribution: ParticleDistribution,
    pub size: RangeInclusive<FCoord>,
    pub size_function: SizeFunction,
    pub velocity: vec2<FCoord>,
    pub lifetime: RangeInclusive<FTime>,
}

#[derive(Debug, Clone, Copy)]
pub enum ParticleKind {
    Mushroom,
    Stun,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SizeFunction {
    #[default]
    Shrink,
    GrowShrink,
}

#[derive(Debug, Clone)]
pub enum ParticleDistribution {
    Circle {
        center: vec2<FCoord>,
        radius: FCoord,
    },
    Aabb(Aabb2<FCoord>),
}

impl ParticleDistribution {
    pub fn sample(&self, rng: &mut impl Rng, density: R32) -> Vec<vec2<FCoord>> {
        match *self {
            ParticleDistribution::Aabb(aabb) => {
                let amount = density * aabb.width() * aabb.height();
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| {
                        vec2(
                            rng.gen_range(aabb.min.x..=aabb.max.x),
                            rng.gen_range(aabb.min.y..=aabb.max.y),
                        )
                    })
                    .collect()
            }
            ParticleDistribution::Circle { center, radius } => {
                let amount = density * radius.sqr() * R32::PI;
                let extra = if rng.gen_bool(amount.fract().as_f32().into()) {
                    1
                } else {
                    0
                };
                let amount = (amount.floor()).as_f32() as usize + extra;

                (0..amount)
                    .map(|_| rng.gen_circle(center, radius))
                    .collect()
            }
        }
    }
}

impl Default for SpawnParticles {
    fn default() -> Self {
        Self {
            kind: ParticleKind::Mushroom,
            density: r32(5.0),
            distribution: ParticleDistribution::Circle {
                center: vec2::ZERO,
                radius: r32(0.5),
            },
            size: r32(0.05)..=r32(0.15),
            size_function: SizeFunction::Shrink,
            velocity: vec2::ZERO,
            lifetime: r32(0.5)..=r32(1.5),
        }
    }
}

pub fn spawn_particles(options: SpawnParticles) -> impl Iterator<Item = Particle> {
    let mut rng = thread_rng();
    options
        .distribution
        .sample(&mut rng, options.density)
        .into_iter()
        .map(move |position| {
            let velocity = rng.gen_circle(options.velocity, r32(0.2));
            let radius = rng.gen_range(options.size.clone());
            let lifetime = rng.gen_range(options.lifetime.clone());
            Particle {
                kind: options.kind,
                position,
                radius,
                size_function: options.size_function,
                velocity,
                lifetime: Bounded::new_max(lifetime),
            }
        })
}
