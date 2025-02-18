use super::{Map, TileType};
use rltk::RGB;

#[must_use]
pub fn tile_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.depth {
        9 | 8 => get_mushroom_glyph(idx, map),
        7 => {
            let x = idx as i32 % map.width;
            if x > map.width - 16 {
                get_tile_glyph_default(idx, map)
            } else {
                get_mushroom_glyph(idx, map)
            }
        }
        5 => {
            let x = idx as i32 % map.width;
            if x < map.width / 2 {
                get_limestone_cavern_glyph(idx, map)
            } else {
                get_tile_glyph_default(idx, map)
            }
        }
        4 | 3 => get_limestone_cavern_glyph(idx, map),
        2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map),
    };

    if map.bloodstains.contains(&idx) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }
    if !map.visible_tiles[idx] {
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // Don't show stains out of visual range
    } else if !map.outdoors {
        fg = fg * map.light[idx];
        bg = bg * map.light[idx];
    }

    (glyph, fg, bg)
}

fn get_forest_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles.get(idx) {
        Some(TileType::Wall) => {
            glyph = rltk::to_cp437('♣');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
        Some(TileType::Bridge) => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        Some(TileType::Road) => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        Some(TileType::Grass) => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        Some(TileType::ShallowWater) => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::CYAN);
        }
        Some(TileType::DeepWater) => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::BLUE);
        }
        Some(TileType::Gravel) => {
            glyph = rltk::to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        Some(TileType::DownStairs) => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        Some(TileType::UpStairs) => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        _ => {
            glyph = rltk::to_cp437('"');
            fg = RGB::from_f32(0.0, 0.5, 0.0);
        }
    }

    (glyph, fg, bg)
}

fn get_mushroom_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles.get(idx) {
        Some(TileType::Wall) => {
            glyph = rltk::to_cp437('♠');
            fg = RGB::from_f32(1.0, 0.0, 1.0);
        }
        Some(TileType::Bridge) => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::GREEN);
        }
        Some(TileType::Road) => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        Some(TileType::Grass) => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        Some(TileType::ShallowWater) => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::CYAN);
        }
        Some(TileType::DeepWater) => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::BLUE);
        }
        Some(TileType::Gravel) => {
            glyph = rltk::to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        Some(TileType::DownStairs) => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        Some(TileType::UpStairs) => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        _ => {
            glyph = rltk::to_cp437('"');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
    }

    (glyph, fg, bg)
}

fn get_limestone_cavern_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles.get(idx) {
        Some(TileType::Wall) => {
            glyph = rltk::to_cp437('▒');
            fg = RGB::from_f32(0.7, 0.7, 0.7);
        }
        Some(TileType::Bridge) => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        Some(TileType::Road) => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        Some(TileType::Grass) => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        Some(TileType::ShallowWater) => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CYAN);
        }
        Some(TileType::DeepWater) => {
            glyph = rltk::to_cp437('▓');
            fg = RGB::from_f32(0.2, 0.2, 1.0);
        }
        Some(TileType::Gravel) => {
            glyph = rltk::to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        Some(TileType::DownStairs) => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        Some(TileType::UpStairs) => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        Some(TileType::Stalactite) => {
            glyph = rltk::to_cp437('╨');
            fg = RGB::from_f32(0.7, 0.7, 0.7);
        }
        Some(TileType::Stalagmite) => {
            glyph = rltk::to_cp437('╥');
            fg = RGB::from_f32(0.7, 0.7, 0.7);
        }
        _ => {
            glyph = rltk::to_cp437('\'');
            fg = RGB::from_f32(0.4, 0.4, 0.4);
        }
    }

    (glyph, fg, bg)
}

fn get_tile_glyph_default(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let bg = RGB::from_f32(0., 0., 0.);

    let (glyph, fg) = match map.tiles.get(idx) {
        Some(TileType::Floor) => (rltk::to_cp437('.'), RGB::from_f32(0.0, 0.5, 0.5)),
        Some(TileType::WoodFloor) => (rltk::to_cp437('░'), RGB::named(rltk::CHOCOLATE)),
        Some(TileType::Wall) => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            (wall_glyph(map, x, y), RGB::from_f32(0., 1.0, 0.))
        }
        Some(TileType::DownStairs) => (rltk::to_cp437('>'), RGB::from_f32(0., 1.0, 1.0)),
        Some(TileType::UpStairs) => (rltk::to_cp437('<'), RGB::from_f32(0., 1.0, 1.0)),
        Some(TileType::Bridge) => (rltk::to_cp437('.'), RGB::named(rltk::CHOCOLATE)),
        Some(TileType::Road) => (rltk::to_cp437('≡'), RGB::named(rltk::GRAY)),
        Some(TileType::Grass) => (rltk::to_cp437('"'), RGB::named(rltk::GREEN)),
        Some(TileType::ShallowWater) => (rltk::to_cp437('~'), RGB::named(rltk::CYAN)),
        Some(TileType::DeepWater) => (rltk::to_cp437('~'), RGB::named(rltk::BLUE)),
        Some(TileType::Gravel) => (rltk::to_cp437(';'), RGB::from_f32(0.5, 0.5, 0.5)),
        Some(TileType::Stalactite) => (rltk::to_cp437('╨'), RGB::from_f32(0.5, 0.5, 0.5)),
        Some(TileType::Stalagmite) => (rltk::to_cp437('╥'), RGB::from_f32(0.5, 0.5, 0.5)),
        _ => (rltk::to_cp437(' '), RGB::named(rltk::BLACK)),
    };

    (glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2_i32 {
        return 35;
    }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,            // Pillar because we can't see neighbors
        1..=3 => 186,      //
        4 | 8 | 12 => 205, //
        5 => 188,          // Wall to the north and west
        6 => 187,          // Wall to the south and west
        7 => 185,          // Wall to the north, south and west
        9 => 200,          // Wall to the north and east
        10 => 201,         // Wall to the south and east
        11 => 204,         // Wall to the north, south and east
        13 => 202,         // Wall to the east, west, and south
        14 => 203,         // Wall to the east, west, and north
        15 => 206,         // ╬ Wall on all sides
        _ => 35,           // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
