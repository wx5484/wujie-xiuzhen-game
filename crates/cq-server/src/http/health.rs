use axum::{extract::State, Json};
use cq_protocol::rest::{ApiOk, HealthResponse};

use crate::{error::ApiError, state::AppState};

pub async fn healthz() -> Json<ApiOk<HealthResponse>> {
    Json(ApiOk::new(HealthResponse {
        status: "ok".into(),
        service: "cq-server".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    }))
}

pub async fn readyz(State(state): State<AppState>) -> Result<Json<ApiOk<HealthResponse>>, ApiError> {
    state.db.ready().await.map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(HealthResponse {
        status: "ready".into(),
        service: "cq-server".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })))
}
