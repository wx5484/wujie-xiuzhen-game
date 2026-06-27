use serde::{Deserialize, Serialize};

use crate::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objectives: serde_json::Value,
    pub rewards: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterQuest {
    pub character_id: EntityId,
    pub quest_id: String,
    pub progress: serde_json::Value,
    pub completed: bool,
}
