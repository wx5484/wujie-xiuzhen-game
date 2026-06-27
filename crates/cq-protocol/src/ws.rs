use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEnvelope {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub seq: u64,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEnvelope {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub seq: u64,
    #[serde(default)]
    pub payload: Value,
}

impl ServerEnvelope {
    pub fn ack(seq: u64, accepted: bool) -> Self {
        Self { msg_type: "ack".into(), seq, payload: json!({ "accepted": accepted }) }
    }

    pub fn event(seq: u64, msg_type: impl Into<String>, payload: impl Serialize) -> Self {
        let payload = serde_json::to_value(payload).unwrap_or_else(|_| json!({}));
        Self { msg_type: msg_type.into(), seq, payload }
    }

    pub fn error(seq: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            msg_type: "error".into(),
            seq,
            payload: json!({ "code": code.into(), "message": message.into() }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub token: String,
    pub character_id: i64,
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub command: String,
    #[serde(default)]
    pub args: Value,
}
