use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasureTemplate {
    pub id: String,
    pub name: String,
    pub family: String,
    pub passive: String,
}

pub fn starter_treasures() -> Vec<TreasureTemplate> {
    vec![
        TreasureTemplate {
            id: "fentian_mark".into(),
            name: "焚天战印".into(),
            family: "输出".into(),
            passive: "攻击时有概率追加火焰伤害。".into(),
        },
        TreasureTemplate {
            id: "xuanwu_core".into(),
            name: "玄武甲心".into(),
            family: "生存".into(),
            passive: "受到伤害时有概率获得护盾。".into(),
        },
    ]
}
