use super::{
    AreaStartingPosition, BuilderChain, BuilderMap, CellularAutomataBuilder, CullUnreachable,
    MetaMapBuilder, TileType, VoronoiSpawning, XStart, YStart,
};
use crate::map;

pub fn forest_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, &"Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Centre, YStart::Middle));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Middle));
    chain.with(VoronoiSpawning::new());
    chain.with(YellowBrickRoad::new());
    chain
}

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        build(build_data);
    }
}

impl YellowBrickRoad {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

fn build(build_data: &mut BuilderMap) {
    let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
    let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);

    let (end_x, end_y) = find_exit(
        build_data,
        build_data.map.width - 2,
        build_data.map.height / 2,
    );
    let end_idx = build_data.map.xy_idx(end_x, end_y);

    build_data.map.populate_blocked();
    let path = rltk::a_star_search(start_idx, end_idx, &build_data.map);
    assert!(path.success, "No valid path for the road");
    for idx in &path.steps {
        let x = *idx as i32 % build_data.map.width;
        let y = *idx as i32 / build_data.map.width;
        paint_road(build_data, x, y);
        paint_road(build_data, x - 1, y);
        paint_road(build_data, x + 1, y);
        paint_road(build_data, x, y - 1);
        paint_road(build_data, x, y + 1);
    }
    build_data.map.tiles[end_idx] = TileType::DownStairs;
    build_data.take_snapshot();

    // Place exit
    let exit_dir = crate::rng::roll_dice(1, 2);
    let (seed_x, seed_y, stream_start_x, stream_start_y) = if exit_dir == 1 {
        (build_data.map.width - 1, 1, 0, build_data.height - 1)
    } else {
        (
            build_data.map.width - 1,
            build_data.height - 1,
            1,
            build_data.height - 1,
        )
    };

    let (stairs_x, stairs_y) = find_exit(build_data, seed_x, seed_y);
    let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);
    build_data.take_snapshot();

    let (stream_x, stream_y) = find_exit(build_data, stream_start_x, stream_start_y);
    let stream_idx = build_data.map.xy_idx(stream_x, stream_y);
    let stream = rltk::a_star_search(stairs_idx, stream_idx, &build_data.map);
    for tile in &stream.steps {
        if build_data.map.tiles[*tile] == TileType::Floor {
            build_data.map.tiles[*tile] = TileType::ShallowWater;
        }
    }
    build_data.map.tiles[stairs_idx] = TileType::DownStairs;
    build_data.take_snapshot();
}

fn find_exit(build_data: &BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
    let mut available_floors: Vec<(usize, f32)> = Vec::new();
    for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
        if map::tile_walkable(*tiletype) {
            available_floors.push((
                idx,
                rltk::DistanceAlg::PythagorasSquared.distance2d(
                    rltk::Point::new(
                        idx as i32 % build_data.map.width,
                        idx as i32 / build_data.map.width,
                    ),
                    rltk::Point::new(seed_x, seed_y),
                ),
            ));
        }
    }
    assert!(!available_floors.is_empty(), "No valid floors to start on");

    available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let end_x = available_floors[0].0 as i32 % build_data.map.width;
    let end_y = available_floors[0].0 as i32 / build_data.map.width;
    (end_x, end_y)
}

fn paint_road(build_data: &mut BuilderMap, x: i32, y: i32) {
    if x < 1 || x > build_data.map.width - 2 || y < 1 || y > build_data.map.height - 2 {
        return;
    }
    let idx = build_data.map.xy_idx(x, y);
    if build_data.map.tiles[idx] != TileType::DownStairs {
        build_data.map.tiles[idx] = TileType::Road;
    }
}
