use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Muted,
    Banned,
}

impl AccountStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Muted => "muted",
            Self::Banned => "banned",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: EntityId,
    pub username: String,
    pub email: Option<String>,
    pub status: AccountStatus,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: EntityId,
    pub account_id: EntityId,
    pub token: String,
    pub expires_at: OffsetDateTime,
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginFailure {
    pub username: String,
    pub ip: String,
    pub failed_at: OffsetDateTime,
}
