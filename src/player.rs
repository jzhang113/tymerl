use super::{Map, Player, Position, RunState, State, TileType, Viewshed};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

fn try_move_player(ecs: &mut World, dx: i32, dy: i32) -> RunState {
    use std::cmp::{max, min};
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&players, &mut positions, &mut viewsheds).join() {
        let dest_index = map.get_index(pos.x + dx, pos.y + dy);

        if map.tiles[dest_index] != TileType::Wall {
            pos.x = min(map.width, max(0, pos.x + dx));
            pos.y = min(map.height, max(0, pos.y + dy));
            viewshed.dirty = true;

            return RunState::Running;
        }
    }

    RunState::AwaitingInput
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(&mut gs.ecs, -1, 0)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(&mut gs.ecs, 1, 0)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(&mut gs.ecs, 0, -1)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(&mut gs.ecs, 0, 1)
            }
            _ => RunState::AwaitingInput,
        },
    }
}
