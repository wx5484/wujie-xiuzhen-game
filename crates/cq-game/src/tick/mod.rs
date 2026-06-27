use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickReport {
    pub at: OffsetDateTime,
    pub online: usize,
    pub respawned_mobs: usize,
    pub saved_characters: usize,
}

pub fn heartbeat(online: usize) -> TickReport {
    TickReport { at: OffsetDateTime::now_utc(), online, respawned_mobs: 0, saved_characters: 0 }
}
