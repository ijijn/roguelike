use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder, TileType};

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl MetaMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        apply_iteration(build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn apply_iteration(build_data: &mut BuilderMap) {
    let mut newtiles = build_data.map.tiles.clone();

    for y in 1..build_data.map.height - 1 {
        for x in 1..build_data.map.width - 1 {
            let idx = build_data.map.xy_idx(x, y);
            let mut neighbors = 0;
            if build_data.map.tiles[idx - 1] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx + 1] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx - (build_data.map.width as usize - 1)] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx - (build_data.map.width as usize + 1)] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx + (build_data.map.width as usize - 1)] == TileType::Wall {
                neighbors += 1;
            }
            if build_data.map.tiles[idx + (build_data.map.width as usize + 1)] == TileType::Wall {
                neighbors += 1;
            }

            if neighbors > 4 || neighbors == 0 {
                newtiles[idx] = TileType::Wall;
            } else {
                newtiles[idx] = TileType::Floor;
            }
        }
    }

    build_data.map.tiles = newtiles;
    build_data.take_snapshot();
}

#[allow(clippy::map_entry)]
fn build(build_data: &mut BuilderMap) {
    // First we completely randomize the map, setting 55% of it to be floor.
    for y in 1..build_data.map.height - 1 {
        for x in 1..build_data.map.width - 1 {
            let roll = crate::rng::roll_dice(1, 100);
            let idx = build_data.map.xy_idx(x, y);
            if roll > 55 {
                build_data.map.tiles[idx] = TileType::Floor;
            } else {
                build_data.map.tiles[idx] = TileType::Wall;
            }
        }
    }
    build_data.take_snapshot();

    // Now we iteratively apply cellular automata rules
    for _i in 0..15 {
        apply_iteration(build_data);
    }
}
