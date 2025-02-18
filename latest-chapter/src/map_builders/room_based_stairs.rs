use super::{BuilderMap, MetaMapBuilder, TileType};
pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl RoomBasedStairs {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    if let Some(rooms) = &build_data.rooms {
        let stairs_position = rooms[rooms.len() - 1].center();
        let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    } else {
        panic!("Room Based Stairs only works after rooms have been created");
    }
}
