use super::{spawner, BuilderMap, MetaMapBuilder};

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl RoomBasedSpawner {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    if let Some(rooms) = &build_data.rooms {
        for room in rooms.iter().skip(1) {
            spawner::spawn_room(
                &build_data.map,
                room,
                build_data.map.depth,
                &mut build_data.spawn_list,
            );
        }
    } else {
        panic!("Room Based Spawning only works after rooms have been created");
    }
}
