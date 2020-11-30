use rltk::RGB;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub symbol: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible: Vec<rltk::Point>,
    pub dirty: bool,
}

#[derive(Component)]
pub struct ActFlag {}

#[derive(Component)]
pub struct Schedulable {
    pub current: i32,
    pub base: i32,
    pub delta: i32,
}
