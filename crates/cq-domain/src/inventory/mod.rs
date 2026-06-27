use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemLocation {
    Bag,
    Warehouse,
    Equipped,
    MailAttachment,
    Consignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: EntityId,
    pub owner_character_id: EntityId,
    pub template_id: String,
    pub quantity: i64,
    pub location: ItemLocation,
    pub slot: Option<String>,
    pub bind: bool,
    pub durability: i32,
    pub extra: serde_json::Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquipmentSlots {
    pub weapon: Option<EntityId>,
    pub chest: Option<EntityId>,
    pub head: Option<EntityId>,
    pub feet: Option<EntityId>,
    pub waist: Option<EntityId>,
    pub neck: Option<EntityId>,
    pub ring_left: Option<EntityId>,
    pub ring_right: Option<EntityId>,
    pub bracelet_left: Option<EntityId>,
    pub bracelet_right: Option<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub bag_used: usize,
    pub bag_limit: usize,
    pub warehouse_used: usize,
    pub equipment: EquipmentSlots,
}

pub fn bag_limit_for_level(_level: i32) -> usize {
    5000
}
