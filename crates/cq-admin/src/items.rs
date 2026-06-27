use cq_domain::item::{ItemStats, Rarity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplatePatch {
    pub id: String,
    pub name: String,
    pub rarity: Rarity,
    pub price: i64,
    pub stats: ItemStats,
}
