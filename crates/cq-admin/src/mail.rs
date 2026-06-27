use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailDraft {
    pub to_character_id: Option<i64>,
    pub title: String,
    pub body: String,
    pub gold: i64,
    pub yuanbao: i64,
}

impl MailDraft {
    pub fn validate(&self) -> bool {
        !self.title.trim().is_empty() && !self.body.trim().is_empty()
    }
}
