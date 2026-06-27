use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobTemplatePatch {
    pub id: String,
    pub name: String,
    pub level: i32,
    pub max_hp: i64,
    pub atk: i64,
    pub def: i64,
    pub boss: bool,
}
