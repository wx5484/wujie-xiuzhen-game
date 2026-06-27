use cq_domain::character::CharacterClass;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiOk<T> {
    pub ok: bool,
    pub data: T,
}

impl<T> ApiOk<T> {
    pub fn new(data: T) -> Self {
        Self { ok: true, data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub ok: bool,
    pub code: String,
    pub message: String,
}

impl ApiErrorBody {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { ok: false, code: code.into(), message: message.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub invite_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub account_id: i64,
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacterRequest {
    pub name: String,
    pub class: CharacterClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}
