use rltk::Rect;
use std::convert::TryInto;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
}

impl Map {
    pub fn get_index(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }
}

pub fn build(width: i32, height: i32) -> Map {
    let dim = (width * height).try_into().unwrap();
    let mut map = Map {
        tiles: vec![TileType::Wall; dim],
        width: width,
        height: height,
    };

    let room = Rect::with_size(10, 10, 10, 10);
    build_room(&room, &mut map);

    map
}

fn build_room(room: &Rect, map: &mut Map) {
    for y in room.y1..=room.y2 {
        for x in room.x1..=room.x2 {
            let index = map.get_index(x, y);
            map.tiles[index] = TileType::Floor;
        }
    }
}
