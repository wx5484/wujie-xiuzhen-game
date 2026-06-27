use cq_db::repositories::admin::AdminRepository;
use cq_protocol::dto::{
    AdminAccountList, AdminAuditLogList, AdminCharacterList, AdminItemTemplateList,
    AdminMailOverview, AdminMobTemplateList, DashboardSummary,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInput {
    pub admin_account_id: Option<i64>,
    pub action: String,
    pub target: String,
    pub detail: serde_json::Value,
}

pub async fn dashboard(repo: &AdminRepository) -> Result<DashboardSummary, AdminError> {
    Ok(repo.dashboard().await?)
}

pub async fn accounts(repo: &AdminRepository) -> Result<AdminAccountList, AdminError> {
    Ok(repo.accounts().await?)
}

pub async fn characters(repo: &AdminRepository) -> Result<AdminCharacterList, AdminError> {
    Ok(repo.characters().await?)
}

pub async fn mail_overview(repo: &AdminRepository) -> Result<AdminMailOverview, AdminError> {
    Ok(repo.mail_overview().await?)
}

pub async fn item_templates(repo: &AdminRepository) -> Result<AdminItemTemplateList, AdminError> {
    Ok(repo.item_templates().await?)
}

pub async fn mob_templates(repo: &AdminRepository) -> Result<AdminMobTemplateList, AdminError> {
    Ok(repo.mob_templates().await?)
}

pub async fn audit_logs(repo: &AdminRepository) -> Result<AdminAuditLogList, AdminError> {
    Ok(repo.audit_logs().await?)
}

pub async fn audit(repo: &AdminRepository, input: AuditInput) -> Result<(), AdminError> {
    repo.audit(input.admin_account_id, &input.action, &input.target, input.detail).await?;
    Ok(())
}
