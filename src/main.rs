use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;
use specs_derive::Component;

mod map;
pub use map::*;
mod player;

pub struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        // input + logic
        player::player_input(self, ctx);

        // map
        let map = self.ecs.fetch::<Map>();
        draw_map(&map, ctx);

        // render
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.symbol);
        }
    }
}

fn draw_map(map: &map::Map, ctx: &mut Rltk) {
    let mut x = 0;
    let mut y = 0;

    for tile in map.tiles.iter() {
        match tile {
            map::TileType::Floor => ctx.set(
                x,
                y,
                RGB::from_f32(0.5, 0.5, 0.5),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('.'),
            ),
            map::TileType::Wall => ctx.set(
                x,
                y,
                RGB::from_f32(0.5, 1.0, 0.0),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('#'),
            ),
        }

        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    symbol: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<player::Player>();

    gs.ecs
        .create_entity()
        .with(Position { x: 10, y: 10 })
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(player::Player {})
        .build();

    let map = map::build(80, 50);
    gs.ecs.insert(map);

    rltk::main_loop(context, gs)
}
