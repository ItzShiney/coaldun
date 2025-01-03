use serde::Deserialize;
use serde::Serialize;

mod state;

pub use state::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Authorized {
    pub player_entity_id: EntityId,
    pub update: ClientUpdate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerSignal {
    ReloadServer,
}
