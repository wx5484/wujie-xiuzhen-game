use cq_domain::mob::MobTemplate;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossRespawn {
    pub template_id: String,
    pub zone: String,
    pub room: String,
    pub killed_at: OffsetDateTime,
    pub respawn_at: OffsetDateTime,
}

pub fn schedule_respawn(template: &MobTemplate, zone: &str, room: &str) -> BossRespawn {
    let now = OffsetDateTime::now_utc();
    BossRespawn {
        template_id: template.id.clone(),
        zone: zone.into(),
        room: room.into(),
        killed_at: now,
        respawn_at: now + Duration::seconds(i64::from(template.respawn_seconds)),
    }
}
