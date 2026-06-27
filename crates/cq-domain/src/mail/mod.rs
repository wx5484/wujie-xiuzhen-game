use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub id: EntityId,
    pub to_character_id: EntityId,
    pub from_name: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub claimed: bool,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailAttachment {
    pub id: EntityId,
    pub mail_id: EntityId,
    pub item_template_id: Option<String>,
    pub quantity: i64,
    pub gold: i64,
    pub yuanbao: i64,
}
