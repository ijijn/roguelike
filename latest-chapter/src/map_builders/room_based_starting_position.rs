use super::{BuilderMap, MetaMapBuilder, Position};

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl RoomBasedStartingPosition {
        pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    if let Some(rooms) = &build_data.rooms {
        let start_pos = rooms[0].center();
        build_data.starting_position = Some(Position {
            x: start_pos.0,
            y: start_pos.1,
        });
    } else {
        panic!("Room Based Staring Position only works after rooms have been created");
    }
}
