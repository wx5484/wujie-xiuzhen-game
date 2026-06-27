use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Realm {
    pub id: i32,
    pub name: String,
    pub open: bool,
    pub max_online: i32,
}

impl Realm {
    pub fn default_realm() -> Self {
        Self { id: 1, name: "一区".into(), open: true, max_online: 500 }
    }
}
