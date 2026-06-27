use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: EntityId,
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub starts_at: Option<OffsetDateTime>,
    pub ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPoints {
    pub character_id: EntityId,
    pub activity_code: String,
    pub points: i64,
}
