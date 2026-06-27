use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SabakCampaign {
    pub id: i64,
    pub signup_starts_at: OffsetDateTime,
    pub battle_starts_at: OffsetDateTime,
    pub battle_ends_at: OffsetDateTime,
    pub defending_guild_id: Option<i64>,
    pub tax_rate_pct: i32,
}

impl SabakCampaign {
    pub fn is_battle_open(&self, now: OffsetDateTime) -> bool {
        now >= self.battle_starts_at && now <= self.battle_ends_at
    }
}
