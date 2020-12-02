use super::DamageEvent;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, DamageEvent>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut damages) = data;

        for (_ent, damage_event) in (&entities, &damages).join() {
            let total_damage = damage_event.amount.iter().sum::<i32>();
            println!("{:?} took {:?}", _ent, total_damage);
        }

        damages.clear();
    }
}
