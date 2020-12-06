#[macro_use]
extern crate lazy_static;

use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

mod components;
pub mod events;
mod gamelog;
mod gui;
mod map;
mod player;
mod sys_particle;
mod sys_turn;
mod sys_visibility;

pub use components::*;
pub use map::{Map, TileType};
pub use sys_particle::ParticleBuilder;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    Running,
}

pub struct State {
    ecs: World,
    tick: i32,
}

impl State {
    fn run_systems(&mut self) {
        self.tick += 1;
        let mut vis = sys_visibility::VisibilitySystem {};
        vis.run_now(&self.ecs);

        events::process_stack(&mut self.ecs);

        let mut turns = sys_turn::TurnSystem {};
        turns.run_now(&self.ecs);

        let mut particles = sys_particle::ParticleSpawnSystem {};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // cleanup
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        sys_particle::cleanup_particles(&mut self.ecs, ctx);

        // draw map + gui
        draw_map(&self.ecs, ctx);
        draw_renderables(&self.ecs, ctx);
        gui::draw_ui(&self.ecs, ctx);

        let mut next_status;
        // wrapping to limit borrowed lifetimes
        {
            let log = self.ecs.fetch::<gamelog::GameLog>();
            for (line, message) in log.entries.iter().rev().take(5).enumerate() {
                ctx.print(2, 50 + line + 1, message);
            }

            let player = self.ecs.fetch::<Entity>();
            let can_act = self.ecs.read_storage::<CanActFlag>();
            match can_act.get(*player) {
                None => ctx.print(30, 1, format!("OPPONENT TURN {}", self.tick)),
                Some(_) => ctx.print(30, 1, format!("YOUR TURN {}", self.tick)),
            }

            // get the current RunState
            next_status = *self.ecs.fetch::<RunState>();
        }

        match next_status {
            RunState::AwaitingInput => {
                next_status = player::player_input(self, ctx);
            }
            RunState::Running => {
                // uncomment while loop to skip rendering intermediate states
                // while next_status == RunState::Running {
                self.run_systems();
                //std::thread::sleep(std::time::Duration::from_millis(100));
                next_status = *self.ecs.fetch::<RunState>();
                //}
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

fn draw_renderables(ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let lifetimes = ecs.read_storage::<ParticleLifetime>();

    for (pos, render, lifetime) in (&positions, &renderables, (&lifetimes).maybe()).join() {
        match lifetime {
            None => ctx.set(pos.x, pos.y, render.fg, render.bg, render.symbol),
            Some(lifetime) => {
                let mut fg = render.fg;
                let mut bg = render.bg;

                if lifetime.should_fade {
                    let fade_percent = ezing::expo_inout(1.0 - lifetime.remaining / lifetime.base);
                    let base_color = RGB::named(rltk::BLACK);

                    fg = fg.lerp(base_color, fade_percent);
                    bg = bg.lerp(base_color, fade_percent);
                }

                ctx.set_active_console(0);
                ctx.set(pos.x, pos.y, fg, bg, render.symbol);
                ctx.set_active_console(1);
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    const WIDTH: i32 = 80;
    const HEIGHT: i32 = 50;
    const CONSOLE_HEIGHT: i32 = HEIGHT + 7;

    let context = RltkBuilder::simple(WIDTH, CONSOLE_HEIGHT)?
        .with_title("Roguelike Tutorial")
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console_no_bg(WIDTH, CONSOLE_HEIGHT, "terminal8x8.png")
        .build()
        .expect("Failed to build console");

    let mut gs = State {
        ecs: World::new(),
        tick: 0,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<CanActFlag>();
    gs.ecs.register::<CanReactFlag>();
    gs.ecs.register::<Schedulable>();
    gs.ecs.register::<ParticleLifetime>();

    gs.ecs.insert(RunState::Running);
    gs.ecs.insert(sys_particle::ParticleBuilder::new());

    let map = map::build_rogue_map(WIDTH, HEIGHT);
    let player_pos = map.rooms[0].center();
    gs.ecs.insert(map);

    let log = gamelog::GameLog {
        entries: vec!["Hello world!".to_string()],
    };
    gs.ecs.insert(log);

    let player = gs
        .ecs
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
        .with(CanReactFlag {})
        .build();
    gs.ecs.insert(player);

    rltk::main_loop(context, gs)
}
