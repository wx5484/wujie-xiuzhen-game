use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAdminCommand {
    pub account_id: i64,
    pub reason: String,
}

pub fn validate_reason(reason: &str) -> bool {
    reason.trim().len() >= 3
}
