use super::{BuilderMap, MetaMapBuilder, Rect, TileType};

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl RoomDrawer {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    let rooms: Vec<Rect>;
    if let Some(rooms_builder) = &build_data.rooms {
        rooms = rooms_builder.clone();
    } else {
        panic!("Room Drawing require a builder with room structures");
    }

    for room in &rooms {
        let room_type = crate::rng::roll_dice(1, 4);
        match room_type {
            1 => circle(build_data, room),
            _ => rectangle(build_data, room),
        }
        build_data.take_snapshot();
    }
}

fn circle(build_data: &mut BuilderMap, room: &Rect) {
    let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
    let center = room.center();
    let center_pt = rltk::Point::new(center.0, center.1);
    for y in room.y1..=room.y2 {
        for x in room.x1..=room.x2 {
            let idx = build_data.map.xy_idx(x, y);
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(center_pt, rltk::Point::new(x, y));
            if idx > 0
                && idx < ((build_data.map.width * build_data.map.height) - 1) as usize
                && distance <= radius
            {
                build_data.map.tiles[idx] = TileType::Floor;
            }
        }
    }
}

fn rectangle(build_data: &mut BuilderMap, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = build_data.map.xy_idx(x, y);
            if idx > 0 && idx < ((build_data.map.width * build_data.map.height) - 1) as usize {
                build_data.map.tiles[idx] = TileType::Floor;
            }
        }
    }
}
