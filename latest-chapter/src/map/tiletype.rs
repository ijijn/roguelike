use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Stalactite,
    Stalagmite,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

#[must_use] pub const fn tile_walkable(tt: TileType) -> bool {
    matches!(
        tt,
        TileType::Floor
            | TileType::DownStairs
            | TileType::Road
            | TileType::Grass
            | TileType::ShallowWater
            | TileType::WoodFloor
            | TileType::Bridge
            | TileType::Gravel
            | TileType::UpStairs
    )
}

#[must_use] pub const fn tile_opaque(tt: TileType) -> bool {
    matches!(
        tt,
        TileType::Wall | TileType::Stalactite | TileType::Stalagmite
    )
}

#[must_use] pub const fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
