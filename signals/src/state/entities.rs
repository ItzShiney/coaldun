use super::AssetId;
use super::Position;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub asset_id: AssetId,
    pub pos: Position,
}
