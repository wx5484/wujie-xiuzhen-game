use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub rank: i32,
    pub character_id: i64,
    pub name: String,
    pub score: i64,
}

pub fn assign_ranks(mut entries: Vec<RankingEntry>) -> Vec<RankingEntry> {
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    for (idx, entry) in entries.iter_mut().enumerate() {
        entry.rank = idx as i32 + 1;
    }
    entries
}
