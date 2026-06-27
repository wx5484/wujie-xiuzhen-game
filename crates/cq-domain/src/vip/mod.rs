use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VipTier {
    Vip,
    Svip,
    PermanentSvip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipRecord {
    pub id: EntityId,
    pub character_id: EntityId,
    pub tier: VipTier,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RechargeCard {
    pub id: EntityId,
    pub code: String,
    pub yuanbao: i64,
    pub used_by: Option<EntityId>,
}
