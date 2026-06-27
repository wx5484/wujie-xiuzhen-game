use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultivationTier {
    pub code: String,
    pub name: String,
    pub min_level: i32,
    pub exp_multiplier: i64,
}

pub fn tiers() -> Vec<CultivationTier> {
    vec![
        CultivationTier { code: "zhuji".into(), name: "筑基".into(), min_level: 60, exp_multiplier: 2 },
        CultivationTier { code: "yuanying".into(), name: "元婴".into(), min_level: 90, exp_multiplier: 4 },
        CultivationTier { code: "dacheng".into(), name: "大乘".into(), min_level: 120, exp_multiplier: 8 },
        CultivationTier { code: "tianxian".into(), name: "天仙".into(), min_level: 150, exp_multiplier: 16 },
    ]
}
