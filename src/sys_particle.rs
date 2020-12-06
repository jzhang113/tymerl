use super::{ParticleLifetime, Position, Renderable};
use rltk::{FontCharType, Point, Rltk, RGB};
use specs::prelude::*;

pub fn cleanup_particles(ecs: &mut World, ctx: &Rltk) {
    let dead_particles = update_lifetimes(ecs, ctx);

    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Failed to delete particle");
    }
}

fn update_lifetimes(ecs: &mut World, ctx: &Rltk) -> Vec<Entity> {
    let mut dead_particles = Vec::new();
    let mut particles = ecs.write_storage::<ParticleLifetime>();
    let entities = ecs.entities();

    for (ent, mut lifetime) in (&entities, &mut particles).join() {
        lifetime.remaining -= ctx.frame_time_ms;
        if lifetime.remaining < 0.0 {
            dead_particles.push(ent);
        }
    }

    dead_particles
}

struct ParticleRequest {
    position: Point,
    color: RGB,
    symbol: FontCharType,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(&mut self, position: Point, color: RGB, symbol: FontCharType, lifetime: f32) {
        self.requests.push(ParticleRequest {
            position,
            color,
            symbol,
            lifetime,
        })
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut lifetimes, mut builder) = data;

        for request in builder.requests.iter() {
            let particle = entities.create();
            positions
                .insert(
                    particle,
                    Position {
                        x: request.position.x,
                        y: request.position.y,
                    },
                )
                .expect("Failed to insert Position for particle");
            renderables
                .insert(
                    particle,
                    Renderable {
                        symbol: request.symbol,
                        fg: request.color,
                        bg: rltk::RGB::named(rltk::BLACK),
                    },
                )
                .expect("Failed to insert Renderable for particle");
            lifetimes
                .insert(
                    particle,
                    ParticleLifetime {
                        base: request.lifetime,
                        remaining: request.lifetime,
                        should_fade: true,
                    },
                )
                .expect("Failed to insert ParticleLifetime for particle");
        }

        builder.requests.clear();
    }
}
