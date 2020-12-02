use rltk::{Point, RGB};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn as_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
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
    pub visible: Vec<Point>,
    pub dirty: bool,
}

#[derive(Component)]
pub struct CanActFlag {}

#[derive(Component)]
pub struct Schedulable {
    pub current: i32,
    pub base: i32,
    pub delta: i32,
}

#[derive(Component)]
pub struct DamageEvent {
    pub amount: Vec<i32>,
}

impl DamageEvent {
    pub fn add_damage(store: &mut WriteStorage<DamageEvent>, ent: Entity, amount: i32) {
        match store.get_mut(ent) {
            None => {
                let event = DamageEvent {
                    amount: vec![amount],
                };
                store.insert(ent, event).expect("Failed to insert damage");
            }
            Some(event) => {
                event.amount.push(amount);
            }
        }
    }
}
