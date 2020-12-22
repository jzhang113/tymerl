use super::{DeathTrigger, Health};
use specs::prelude::*;

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, DeathTrigger>,
        ReadStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player, death_triggers, healths) = data;
        let mut dead = Vec::new();

        for (ent, health, effect) in (&entities, &healths, (&death_triggers).maybe()).join() {
            if health.current <= 0 {
                if let Some(effect) = effect {
                    super::add_event(effect.prototype, vec![], true);
                }

                if ent != *player {
                    dead.push(ent);
                } else {
                    println!("You are dead");
                }
            }
        }

        for victim in dead {
            entities
                .delete(victim)
                .expect("Failed to remove dead entity");
        }
    }
}
