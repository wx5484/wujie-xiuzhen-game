use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfkSession {
    pub character_id: i64,
    pub started_at: OffsetDateTime,
    pub last_settled_at: OffsetDateTime,
    pub exp_per_minute: i64,
    pub gold_per_minute: i64,
}

impl AfkSession {
    pub fn settle(&mut self, now: OffsetDateTime) -> (i64, i64) {
        let minutes = (now - self.last_settled_at).whole_minutes().max(0);
        self.last_settled_at = now;
        (minutes * self.exp_per_minute, minutes * self.gold_per_minute)
    }
}
