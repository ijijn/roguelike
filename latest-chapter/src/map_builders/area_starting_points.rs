use super::{BuilderMap, MetaMapBuilder, Position};
use crate::map;

#[allow(dead_code)]
pub enum XStart {
    Left,
    Centre,
    Right,
}

#[allow(dead_code)]
pub enum YStart {
    Top,
    Middle,
    Bottom,
}

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl AreaStartingPosition {
    #[allow(dead_code)]
    pub fn new(x: XStart, y: YStart) -> Box<Self> {
        Box::new(Self { x, y })
    }

    fn build(&self, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XStart::Left => 1,
            XStart::Centre => build_data.map.width / 2,
            XStart::Right => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YStart::Top => 1,
            YStart::Middle => build_data.map.height / 2,
            YStart::Bottom => build_data.map.height - 2,
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

        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;

        build_data.starting_position = Some(Position {
            x: start_x,
            y: start_y,
        });
    }
}
