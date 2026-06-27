use cq_domain::{
    character::{Character, CharacterState, CharacterStats},
    map::Room,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateEvent {
    pub character: Character,
    pub stats: CharacterStats,
    pub state: CharacterState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStateEvent {
    pub room: Room,
    pub players: Vec<String>,
    pub mobs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLogEvent {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNoticeEvent {
    pub level: String,
    pub message: String,
}
