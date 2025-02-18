use super::{BuilderMap, MetaMapBuilder, TileType};
use crate::map;

pub enum XEnd {
    Left,
    Centre,
    Right,
}

pub enum YEnd {
    Top,
    Middle,
    Bottom,
}

pub struct AreaEndingPosition {
    x: XEnd,
    y: YEnd,
}

impl MetaMapBuilder for AreaEndingPosition {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl AreaEndingPosition {
    pub fn new(x: XEnd, y: YEnd) -> Box<Self> {
        Box::new(Self { x, y })
    }

    fn build(&self, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XEnd::Left => 1,
            XEnd::Centre => build_data.map.width / 2,
            XEnd::Right => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YEnd::Top => 1,
            YEnd::Middle => build_data.map.height / 2,
            YEnd::Bottom => build_data.map.height - 2,
        };

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

        build_data.map.tiles[available_floors[0].0] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
