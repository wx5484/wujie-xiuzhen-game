use serde::{Deserialize, Serialize};

use crate::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: EntityId,
    pub owner_character_id: EntityId,
    pub template_id: String,
    pub name: String,
    pub level: i32,
    pub exp: i64,
    pub fighting: bool,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetTemplate {
    pub id: String,
    pub name: String,
    pub rarity: String,
    pub base_hp: i64,
    pub base_atk: i64,
}
