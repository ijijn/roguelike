use super::{spawner, BuilderMap, MetaMapBuilder};

pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl CorridorSpawner {
        pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    if let Some(corridors) = &build_data.corridors {
        for c in corridors {
            let depth = build_data.map.depth;
            spawner::spawn_region(&build_data.map, c, depth, &mut build_data.spawn_list);
        }
    } else {
        panic!("Corridor Based Spawning only works after corridors have been created");
    }
}
