use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeStatus {
    Open,
    Locked,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: EntityId,
    pub from_character_id: EntityId,
    pub to_character_id: EntityId,
    pub status: TradeStatus,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsignmentStatus {
    Listed,
    Sold,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consignment {
    pub id: EntityId,
    pub seller_character_id: EntityId,
    pub item_id: EntityId,
    pub price_yuanbao: i64,
    pub status: ConsignmentStatus,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}
