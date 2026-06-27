use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomPatch {
    pub zone: String,
    pub room: String,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub spawns: Option<Vec<String>>,
}
