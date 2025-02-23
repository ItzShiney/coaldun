use super::{AssetId, Position};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub asset_id: AssetId,
    pub pos: Position,
}
