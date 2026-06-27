use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPatch {
    pub character_id: i64,
    pub gold_delta: i64,
    pub yuanbao_delta: i64,
    pub reason: String,
}

impl CharacterPatch {
    pub fn is_noop(&self) -> bool {
        self.gold_delta == 0 && self.yuanbao_delta == 0
    }
}
