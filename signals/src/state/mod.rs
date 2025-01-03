use serde::Deserialize;
use serde::Serialize;

mod assets;
mod entities;

pub use assets::*;
pub use entities::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub asset_id: AssetId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientUpdate {
    pub assets: Box<[(AssetId, AssetData)]>,
    pub floors: Box<[(Position, Tile)]>,
    pub walls: Box<[(Position, Tile)]>,
    pub entities: Box<[(EntityId, Entity)]>,
}
