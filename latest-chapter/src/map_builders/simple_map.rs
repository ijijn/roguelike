use super::{BuilderMap, InitialMapBuilder, Rect};

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build_rooms(build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build_rooms(build_data: &mut BuilderMap) {
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;
    let mut rooms: Vec<Rect> = Vec::new();

    for _i in 0..MAX_ROOMS {
        let w = crate::rng::range(MIN_SIZE, MAX_SIZE);
        let h = crate::rng::range(MIN_SIZE, MAX_SIZE);
        let x = crate::rng::roll_dice(1, build_data.map.width - w - 1) - 1;
        let y = crate::rng::roll_dice(1, build_data.map.height - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in &rooms {
            if new_room.intersect(other_room) {
                ok = false;
            }
        }
        if ok {
            rooms.push(new_room);
        }
    }
    build_data.rooms = Some(rooms);
}
