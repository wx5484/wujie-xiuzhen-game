use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuildRole {
    Leader,
    Elder,
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: EntityId,
    pub name: String,
    pub notice: String,
    pub level: i32,
    pub funds: i64,
    pub sabak_owner: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMember {
    pub guild_id: EntityId,
    pub character_id: EntityId,
    pub role: GuildRole,
    pub contribution: i64,
    pub joined_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildApplication {
    pub id: EntityId,
    pub guild_id: EntityId,
    pub character_id: EntityId,
    pub message: String,
    pub created_at: OffsetDateTime,
}
