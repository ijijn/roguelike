use super::{BuilderChain, BuilderMap, InitialMapBuilder, Position, TileType};
use std::collections::HashSet;

pub fn town_builder(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, &"The Town of Bracketon");
    chain.start_with(TownBuilder::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
        fn build_map(&mut self, build_data: &mut BuilderMap) {
        build_rooms(build_data);
    }
}

enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}

impl TownBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

pub fn build_rooms(build_data: &mut BuilderMap) {
    grass_layer(build_data);
    water_and_piers(build_data);
    let (mut available_building_tiles, wall_gap_y) = town_walls(build_data);
    let buildings = buildings(build_data, &mut available_building_tiles);
    let doors = add_doors(build_data, &buildings, wall_gap_y);
    add_paths(build_data, &doors);

    for y in wall_gap_y - 3..wall_gap_y + 4 {
        let exit_idx = build_data.map.xy_idx(build_data.width - 2, y);
        build_data.map.tiles[exit_idx] = TileType::DownStairs;
    }

    let building_size = sort_buildings(&buildings);
    building_factory(build_data, &buildings, &building_size);

    spawn_dockers(build_data);
    spawn_townsfolk(build_data, &available_building_tiles);

    // Make visible for screenshot
    for t in &mut build_data.map.visible_tiles {
        *t = true;
    }
    build_data.take_snapshot();
}

fn building_factory(
    build_data: &mut BuilderMap,
    buildings: &[(i32, i32, i32, i32)],
    building_index: &[(usize, i32, BuildingTag)],
) {
    for (i, building) in buildings.iter().enumerate() {
        let build_type = &building_index[i].2;
        match build_type {
            BuildingTag::Pub => build_pub(building, build_data),
            BuildingTag::Temple => build_temple(building, build_data),
            BuildingTag::Blacksmith => build_smith(building, build_data),
            BuildingTag::Clothier => build_clothier(building, build_data),
            BuildingTag::Alchemist => build_alchemist(building, build_data),
            BuildingTag::PlayerHouse => build_my_house(building, build_data),
            BuildingTag::Hovel => build_hovel(building, build_data),
            BuildingTag::Abandoned => build_abandoned_house(building, build_data),
            BuildingTag::Unassigned => {}
        }
    }
}

fn spawn_dockers(build_data: &mut BuilderMap) {
    for (idx, tt) in build_data.map.tiles.iter().enumerate() {
        if *tt == TileType::Bridge && crate::rng::roll_dice(1, 6) == 1 {
            let roll = crate::rng::roll_dice(1, 3);
            match roll {
                1 => build_data.spawn_list.push((idx, "Dock Worker".to_string())),
                2 => build_data
                    .spawn_list
                    .push((idx, "Wannabe Pirate".to_string())),
                _ => build_data.spawn_list.push((idx, "Fisher".to_string())),
            }
        }
    }
}

fn spawn_townsfolk(build_data: &mut BuilderMap, available_building_tiles: &HashSet<usize>) {
    for idx in available_building_tiles {
        if crate::rng::roll_dice(1, 10) == 1 {
            let roll = crate::rng::roll_dice(1, 4);
            match roll {
                1 => build_data.spawn_list.push((*idx, "Peasant".to_string())),
                2 => build_data.spawn_list.push((*idx, "Drunk".to_string())),
                3 => build_data
                    .spawn_list
                    .push((*idx, "Dock Worker".to_string())),
                _ => build_data.spawn_list.push((*idx, "Fisher".to_string())),
            }
        }
    }
}

fn grass_layer(build_data: &mut BuilderMap) {
    // We'll start with a nice layer of grass
    for t in &mut build_data.map.tiles {
        *t = TileType::Grass;
    }
    build_data.take_snapshot();
}

fn water_and_piers(build_data: &mut BuilderMap) {
    let mut n = (crate::rng::roll_dice(1, 65535) as f32) / 65535f32;
    let mut water_width: Vec<i32> = Vec::new();
    for y in 0..build_data.height {
        let n_water = (f32::sin(n) * 10.0) as i32 + 14 + crate::rng::roll_dice(1, 6);
        water_width.push(n_water);
        n += 0.1;
        for x in 0..n_water {
            let idx = build_data.map.xy_idx(x, y);
            build_data.map.tiles[idx] = TileType::DeepWater;
        }
        for x in n_water..n_water + 3 {
            let idx = build_data.map.xy_idx(x, y);
            build_data.map.tiles[idx] = TileType::ShallowWater;
        }
    }
    build_data.take_snapshot();

    // Add piers
    for _i in 0..crate::rng::roll_dice(1, 4) + 6 {
        let y = crate::rng::roll_dice(1, build_data.height) - 1;
        for x in 2 + crate::rng::roll_dice(1, 6)..water_width[y as usize] + 4 {
            let idx = build_data.map.xy_idx(x, y);
            build_data.map.tiles[idx] = TileType::Bridge;
        }
    }
    build_data.take_snapshot();
}

fn town_walls(build_data: &mut BuilderMap) -> (HashSet<usize>, i32) {
    let mut available_building_tiles: HashSet<usize> = HashSet::new();
    let wall_gap_y = crate::rng::roll_dice(1, build_data.height - 9) + 5;
    for y in 1..build_data.height - 2 {
        if y > wall_gap_y - 4 && y < wall_gap_y + 4 {
            for x in 30..build_data.width {
                let road_idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[road_idx] = TileType::Road;
            }
        } else {
            let idx = build_data.map.xy_idx(30, y);
            build_data.map.tiles[idx] = TileType::Wall;
            build_data.map.tiles[idx - 1] = TileType::Floor;
            let idx_right = build_data.map.xy_idx(build_data.width - 2, y);
            build_data.map.tiles[idx_right] = TileType::Wall;
            for x in 31..build_data.width - 2 {
                let gravel_idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[gravel_idx] = TileType::Gravel;
                if y > 2 && y < build_data.height - 1 {
                    available_building_tiles.insert(gravel_idx);
                }
            }
        }
    }
    build_data.take_snapshot();

    for x in 30..build_data.width - 1 {
        let idx_top = build_data.map.xy_idx(x, 1);
        build_data.map.tiles[idx_top] = TileType::Wall;
        let idx_bot = build_data.map.xy_idx(x, build_data.height - 2);
        build_data.map.tiles[idx_bot] = TileType::Wall;
    }
    build_data.take_snapshot();

    (available_building_tiles, wall_gap_y)
}

fn buildings(
    build_data: &mut BuilderMap,
    available_building_tiles: &mut HashSet<usize>,
) -> Vec<(i32, i32, i32, i32)> {
    let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
    let mut n_buildings = 0;
    while n_buildings < 12 {
        let bx = crate::rng::roll_dice(1, build_data.map.width - 32) + 30;
        let by = crate::rng::roll_dice(1, build_data.map.height) - 2;
        let bw = crate::rng::roll_dice(1, 8) + 4;
        let bh = crate::rng::roll_dice(1, 8) + 4;
        let mut possible = true;
        for y in by..by + bh {
            for x in bx..bx + bw {
                if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                    possible = false;
                } else {
                    let idx = build_data.map.xy_idx(x, y);
                    if !available_building_tiles.contains(&idx) {
                        possible = false;
                    }
                }
            }
        }
        if possible {
            n_buildings += 1;
            buildings.push((bx, by, bw, bh));
            for y in by..by + bh {
                for x in bx..bx + bw {
                    let idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[idx] = TileType::WoodFloor;
                    available_building_tiles.remove(&idx);
                    available_building_tiles.remove(&(idx + 1));
                    available_building_tiles.remove(&(idx + build_data.width as usize));
                    available_building_tiles.remove(&(idx - 1));
                    available_building_tiles.remove(&(idx - build_data.width as usize));
                }
            }
            build_data.take_snapshot();
        }
    }

    // Outline buildings
    let mut mapclone = build_data.map.clone();
    for y in 2..build_data.height - 2 {
        for x in 32..build_data.width - 2 {
            let idx = build_data.map.xy_idx(x, y);
            if build_data.map.tiles[idx] == TileType::WoodFloor {
                let mut neighbors = 0;
                if build_data.map.tiles[idx - 1] != TileType::WoodFloor {
                    neighbors += 1;
                }
                if build_data.map.tiles[idx + 1] != TileType::WoodFloor {
                    neighbors += 1;
                }
                if build_data.map.tiles[idx - build_data.width as usize] != TileType::WoodFloor {
                    neighbors += 1;
                }
                if build_data.map.tiles[idx + build_data.width as usize] != TileType::WoodFloor {
                    neighbors += 1;
                }
                if neighbors > 0 {
                    mapclone.tiles[idx] = TileType::Wall;
                }
            }
        }
    }
    build_data.map = mapclone;
    build_data.take_snapshot();
    buildings
}

fn add_doors(
    build_data: &mut BuilderMap,
    buildings: &[(i32, i32, i32, i32)],
    wall_gap_y: i32,
) -> Vec<usize> {
    let mut doors = Vec::new();
    for building in buildings {
        let door_x = building.0 + 1 + crate::rng::roll_dice(1, building.2 - 3);
        let cy = building.1 + (building.3 / 2);
        let idx = if cy > wall_gap_y {
            // Door on the north wall
            build_data.map.xy_idx(door_x, building.1)
        } else {
            build_data.map.xy_idx(door_x, building.1 + building.3 - 1)
        };
        build_data.map.tiles[idx] = TileType::Floor;
        build_data.spawn_list.push((idx, "Door".to_string()));
        doors.push(idx);
    }
    build_data.take_snapshot();
    doors
}

fn add_paths(build_data: &mut BuilderMap, doors: &[usize]) {
    let mut roads = Vec::new();
    for y in 0..build_data.height {
        for x in 0..build_data.width {
            let idx = build_data.map.xy_idx(x, y);
            if build_data.map.tiles[idx] == TileType::Road {
                roads.push(idx);
            }
        }
    }

    build_data.map.populate_blocked();
    for door_idx in doors {
        let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
        let door_pt = rltk::Point::new(
            *door_idx as i32 % build_data.map.width,
            *door_idx as i32 / build_data.map.width,
        );
        for r in &roads {
            nearest_roads.push((
                *r,
                rltk::DistanceAlg::PythagorasSquared.distance2d(
                    door_pt,
                    rltk::Point::new(
                        *r as i32 % build_data.map.width,
                        *r as i32 / build_data.map.width,
                    ),
                ),
            ));
        }
        nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let destination = nearest_roads[0].0;
        let path = rltk::a_star_search(*door_idx, destination, &build_data.map);
        if path.success {
            for step in &path.steps {
                let idx = { *step };
                build_data.map.tiles[idx] = TileType::Road;
                roads.push(idx);
            }
        }
        build_data.take_snapshot();
    }
}

fn sort_buildings(buildings: &[(i32, i32, i32, i32)]) -> Vec<(usize, i32, BuildingTag)> {
    let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
    for (i, building) in buildings.iter().enumerate() {
        building_size.push((i, building.2 * building.3, BuildingTag::Unassigned));
    }
    building_size.sort_by(|a, b| b.1.cmp(&a.1));
    building_size[0].2 = BuildingTag::Pub;
    building_size[1].2 = BuildingTag::Temple;
    building_size[2].2 = BuildingTag::Blacksmith;
    building_size[3].2 = BuildingTag::Clothier;
    building_size[4].2 = BuildingTag::Alchemist;
    building_size[5].2 = BuildingTag::PlayerHouse;
    for b in building_size.iter_mut().skip(6) {
        b.2 = BuildingTag::Hovel;
    }
    let last_index = building_size.len() - 1;
    building_size[last_index].2 = BuildingTag::Abandoned;
    building_size
}

fn random_building_spawn(
    building: &(i32, i32, i32, i32),
    build_data: &mut BuilderMap,
    to_place: &mut Vec<&str>,
    player_idx: usize,
) {
    for y in building.1..building.1 + building.3 {
        for x in building.0..building.0 + building.2 {
            let idx = build_data.map.xy_idx(x, y);
            if build_data.map.tiles[idx] == TileType::WoodFloor
                && idx != player_idx
                && crate::rng::roll_dice(1, 3) == 1
                && !to_place.is_empty()
            {
                let entity_tag = to_place[0];
                to_place.remove(0);
                build_data.spawn_list.push((idx, entity_tag.to_string()));
            }
        }
    }
}

fn build_pub(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place the player
    build_data.starting_position = Some(Position {
        x: building.0 + (building.2 / 2),
        y: building.1 + (building.3 / 2),
    });
    let player_idx = build_data
        .map
        .xy_idx(building.0 + (building.2 / 2), building.1 + (building.3 / 2));

    // Place other items
    let mut to_place: Vec<&str> = vec![
        "Barkeep",
        "Shady Salesman",
        "Patron",
        "Patron",
        "Keg",
        "Table",
        "Chair",
        "Table",
        "Chair",
    ];
    random_building_spawn(building, build_data, &mut to_place, player_idx);
}

fn build_temple(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> = vec![
        "Priest",
        "Altar",
        "Parishioner",
        "Parishioner",
        "Chair",
        "Chair",
        "Candle",
        "Candle",
    ];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_smith(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> = vec![
        "Blacksmith",
        "Anvil",
        "Water Trough",
        "Weapon Rack",
        "Armor Stand",
    ];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_clothier(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> = vec!["Clothier", "Cabinet", "Table", "Loom", "Hide Rack"];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_alchemist(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> =
        vec!["Alchemist", "Chemistry Set", "Dead Thing", "Chair", "Table"];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_my_house(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> = vec!["Mom", "Bed", "Cabinet", "Chair", "Table"];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_hovel(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    // Place items
    let mut to_place: Vec<&str> = vec!["Peasant", "Bed", "Chair", "Table"];
    random_building_spawn(building, build_data, &mut to_place, 0);
}

fn build_abandoned_house(building: &(i32, i32, i32, i32), build_data: &mut BuilderMap) {
    for y in building.1..building.1 + building.3 {
        for x in building.0..building.0 + building.2 {
            let idx = build_data.map.xy_idx(x, y);
            if build_data.map.tiles[idx] == TileType::WoodFloor
                && idx != 0
                && crate::rng::roll_dice(1, 2) == 1
            {
                build_data.spawn_list.push((idx, "Rat".to_string()));
            }
        }
    }
}
