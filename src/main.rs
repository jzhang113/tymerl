use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
mod gamelog;
mod gui;
mod map;
mod player;
mod sys_turn;
mod sys_visibility;

pub use components::*;
pub use map::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    Running,
}

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = sys_visibility::VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut turns = sys_turn::TurnSystem {};
        turns.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // draw map + gui
        ctx.cls();
        draw_map(&self.ecs, ctx);
        gui::draw_ui(&self.ecs, ctx);

        let mut next_status;
        // wrapping to limit borrowed lifetimes
        {
            // render
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            for (pos, render) in (&positions, &renderables).join() {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.symbol);
            }

            let log = self.ecs.fetch::<gamelog::GameLog>();
            for (line, message) in log.entries.iter().rev().take(5).enumerate() {
                ctx.print(2, 50 + line + 1, message);
            }

            // get the current RunState
            next_status = *self.ecs.fetch::<RunState>();
        }

        match next_status {
            RunState::AwaitingInput => {
                next_status = player::player_input(self, ctx);
            }
            RunState::Running => {
                while next_status == RunState::Running {
                    self.run_systems();
                    next_status = *self.ecs.fetch::<RunState>();
                }
            }
        }

        let mut status_writer = self.ecs.write_resource::<RunState>();
        *status_writer = next_status;
    }
}

fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.known_tiles[idx] {
            let symbol;
            let mut fg;

            match tile {
                TileType::Floor => {
                    symbol = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    symbol = rltk::to_cp437('#');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }

            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), symbol);
        }

        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    const WIDTH: i32 = 80;
    const HEIGHT: i32 = 50;

    let context = RltkBuilder::simple(WIDTH, HEIGHT + 7)
        .expect("Failed to create console")
        .with_title("Roguelike Tutorial")
        .build()
        .expect("Failed to build console");

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<ActFlag>();
    gs.ecs.register::<Schedulable>();

    gs.ecs.insert(RunState::Running);

    let map = map::build_rogue_map(WIDTH, HEIGHT);
    let player_pos = map.rooms[0].center();
    gs.ecs.insert(map);

    let log = gamelog::GameLog {
        entries: vec!["Hello world!".to_string()],
    };
    gs.ecs.insert(log);

    gs.ecs
        .create_entity()
        .with(Position {
            x: player_pos.x,
            y: player_pos.y,
        })
        .with(Renderable {
            symbol: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Schedulable {
            current: 0,
            base: 24,
            delta: 4,
        })
        .with(Viewshed {
            visible: Vec::new(),
            dirty: true,
        })
        .build();

    rltk::main_loop(context, gs)
}
