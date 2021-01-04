use super::{AttackIntent, CanActFlag, Map, MoveIntent, Position, Viewshed};
use rltk::Algorithm2D;
use specs::prelude::*;

pub struct AiSystem;

impl<'a> System<'a> for AiSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CanActFlag>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MoveIntent>,
        WriteStorage<'a, AttackIntent>,
        ReadStorage<'a, Viewshed>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut can_act, positions, mut moves, mut attacks, viewsheds, map, player) =
            data;
        let mut turn_done = Vec::new();
        let player_pos = positions.get(*player).unwrap();

        for (ent, _turn, pos, viewshed) in (&entities, &can_act, &positions, &viewsheds).join() {
            if ent == *player {
                // player turn, handled in player.rs
                continue;
            }

            println!("ai turn: {:?}", ent);

            if viewshed
                .visible
                .iter()
                .any(|pos| pos.x == player_pos.x && pos.y == player_pos.y)
            {
                // if we can see the player move towards them
                let curr_index = map.get_index(pos.x, pos.y);
                let player_index = map.get_index(player_pos.x, player_pos.y);
                let path = rltk::a_star_search(curr_index, player_index, &*map);
                let next_pos = map.index_to_point2d(path.steps[1]);
                let movement = MoveIntent { loc: next_pos };
                moves.insert(ent, movement).expect("something");
            } else {
                // else wait for now
            }

            turn_done.push(ent);
        }

        for done in turn_done.iter() {
            can_act.remove(*done);
        }
    }
}
