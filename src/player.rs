use super::{Map, Position, State, TileType};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Player {}

fn try_move_player(ecs: &mut World, dx: i32, dy: i32) {
    use std::cmp::{max, min};
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let dest_index = map.get_index(pos.x + dx, pos.y + dy);

        if map.tiles[dest_index] != TileType::Wall {
            pos.x = min(map.width, max(0, pos.x + dx));
            pos.y = min(map.height, max(0, pos.y + dy));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(&mut gs.ecs, -1, 0),
            VirtualKeyCode::Right => try_move_player(&mut gs.ecs, 1, 0),
            VirtualKeyCode::Up => try_move_player(&mut gs.ecs, 0, -1),
            VirtualKeyCode::Down => try_move_player(&mut gs.ecs, 0, 1),
            _ => {}
        },
    }
}
