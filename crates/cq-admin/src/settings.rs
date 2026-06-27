use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingPatch {
    pub key: String,
    pub value: serde_json::Value,
    pub version: i64,
}
