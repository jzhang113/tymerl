use super::{Map, Position, Renderable};
use rltk::{Algorithm2D, Rltk, RGB};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        50,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let log = ecs.fetch::<super::gamelog::GameLog>();
    for (line, message) in log.entries.iter().rev().take(5).enumerate() {
        ctx.print(2, 50 + line + 1, message);
    }

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let renderables = ecs.read_storage::<Renderable>();
    let positions = ecs.read_storage::<Position>();

    let mouse_point = ctx.mouse_point();
    if !map.in_bounds(mouse_point) {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();

    for (rend, pos) in (&renderables, &positions).join() {
        if pos.as_point() == mouse_point {
            tooltip.push(rend.symbol.to_string());
        }
    }

    if !tooltip.is_empty() {
        // placeholder
        ctx.print_color(
            1,
            1,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::GREY),
            tooltip.first().unwrap(),
        );
    }
}
