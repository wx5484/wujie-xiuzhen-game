use cq_domain::map::Position;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("unknown command")]
    Unknown,
    #[error("missing argument: {0}")]
    MissingArgument(&'static str),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameCommand {
    Move { direction: String },
    Attack { target_id: i64 },
    CastSkill { skill_id: String, target_id: i64 },
    UseItem { item_id: i64 },
    LearnSkill { skill_id: String },
    StateRequest,
}

impl GameCommand {
    pub fn parse(command: &str, args: &Value) -> Result<Self, CommandError> {
        match command {
            "move" => Ok(Self::Move {
                direction: args
                    .get("direction")
                    .and_then(Value::as_str)
                    .ok_or(CommandError::MissingArgument("direction"))?
                    .to_string(),
            }),
            "attack" => Ok(Self::Attack {
                target_id: args
                    .get("target_id")
                    .and_then(Value::as_i64)
                    .ok_or(CommandError::MissingArgument("target_id"))?,
            }),
            "cast_skill" | "skill" => Ok(Self::CastSkill {
                skill_id: args
                    .get("skill_id")
                    .and_then(Value::as_str)
                    .ok_or(CommandError::MissingArgument("skill_id"))?
                    .to_string(),
                target_id: args.get("target_id").and_then(Value::as_i64).unwrap_or(0),
            }),
            "use_item" => Ok(Self::UseItem {
                item_id: args
                    .get("item_id")
                    .and_then(Value::as_i64)
                    .ok_or(CommandError::MissingArgument("item_id"))?,
            }),
            "learn_skill" => Ok(Self::LearnSkill {
                skill_id: args
                    .get("skill_id")
                    .and_then(Value::as_str)
                    .ok_or(CommandError::MissingArgument("skill_id"))?
                    .to_string(),
            }),
            "state" | "state_request" => Ok(Self::StateRequest),
            _ => Err(CommandError::Unknown),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutcome {
    pub position: Option<Position>,
    pub logs: Vec<String>,
    pub state_changed: bool,
}

impl CommandOutcome {
    pub fn log(line: impl Into<String>) -> Self {
        Self { position: None, logs: vec![line.into()], state_changed: true }
    }
}
