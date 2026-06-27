use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub reason: String,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPlan {
    pub file_name: String,
    pub requested_at: OffsetDateTime,
    pub notes: Vec<String>,
}

pub fn plan_backup(request: BackupRequest) -> BackupPlan {
    let ts = OffsetDateTime::now_utc().unix_timestamp();
    BackupPlan {
        file_name: format!("cq-backup-{ts}.dump"),
        requested_at: OffsetDateTime::now_utc(),
        notes: vec![
            format!("reason={}", request.reason),
            format!("requested_by={}", request.requested_by),
            "use pg_dump with DATABASE_URL credentials".into(),
        ],
    }
}
