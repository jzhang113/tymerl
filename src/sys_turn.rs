use super::{ActFlag, Player, Position, RunState, Schedulable};
use specs::prelude::*;

pub struct TurnSystem {}

impl<'a> System<'a> for TurnSystem {
    type SystemData = (
        WriteExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, ActFlag>,
        WriteStorage<'a, Schedulable>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut game_state, entities, mut can_act, mut schedulables, pos, player) = data;
        assert!(*game_state == RunState::Running);

        can_act.clear();

        for (ent, sched, _pos) in (&entities, &mut schedulables, &pos).join() {
            sched.current -= sched.delta;
            if sched.current > 0 {
                continue;
            }

            sched.current += sched.base;
            can_act
                .insert(ent, ActFlag {})
                .expect("Failed to insert ActFlag");

            match player.get(ent) {
                None => {}
                Some(_) => *game_state = RunState::AwaitingInput,
            }
        }
    }
}
