use super::{BuilderMap, MetaMapBuilder, Rect, TileType};

pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl RoomCornerRounder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    let rooms: Vec<Rect>;
    if let Some(rooms_builder) = &build_data.rooms {
        rooms = rooms_builder.clone();
    } else {
        panic!("Room Rounding require a builder with room structures");
    }

    for room in &rooms {
        fill_if_corner(room.x1 + 1, room.y1 + 1, build_data);
        fill_if_corner(room.x2, room.y1 + 1, build_data);
        fill_if_corner(room.x1 + 1, room.y2, build_data);
        fill_if_corner(room.x2, room.y2, build_data);

        build_data.take_snapshot();
    }
}

fn fill_if_corner(x: i32, y: i32, build_data: &mut BuilderMap) {
    let w = build_data.map.width;
    let h = build_data.map.height;
    let idx = build_data.map.xy_idx(x, y);
    let mut neighbor_walls = 0;
    if x > 0 && build_data.map.tiles[idx - 1] == TileType::Wall {
        neighbor_walls += 1;
    }
    if y > 0 && build_data.map.tiles[idx - w as usize] == TileType::Wall {
        neighbor_walls += 1;
    }
    if x < w - 2 && build_data.map.tiles[idx + 1] == TileType::Wall {
        neighbor_walls += 1;
    }
    if y < h - 2 && build_data.map.tiles[idx + w as usize] == TileType::Wall {
        neighbor_walls += 1;
    }

    if neighbor_walls == 2 {
        build_data.map.tiles[idx] = TileType::Wall;
    }
}
