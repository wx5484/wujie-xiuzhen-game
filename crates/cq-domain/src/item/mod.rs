use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Currency,
    Consumable,
    Material,
    Weapon,
    Armor,
    Accessory,
    Book,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Supreme,
    Ultimate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemStats {
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
    pub dex: i64,
    pub spirit: i64,
    pub hp: i64,
    pub mp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub slot: Option<String>,
    pub rarity: Rarity,
    pub price: i64,
    pub stackable: bool,
    pub stats: ItemStats,
}

pub fn starter_templates() -> Vec<ItemTemplate> {
    vec![
        ItemTemplate {
            id: "potion_small".into(),
            name: "生命药剂".into(),
            kind: ItemKind::Consumable,
            slot: None,
            rarity: Rarity::Common,
            price: 1000,
            stackable: true,
            stats: ItemStats { hp: 1000, ..ItemStats::default() },
        },
        ItemTemplate {
            id: "potion_mana".into(),
            name: "魔法药剂".into(),
            kind: ItemKind::Consumable,
            slot: None,
            rarity: Rarity::Common,
            price: 1000,
            stackable: true,
            stats: ItemStats { mp: 1000, ..ItemStats::default() },
        },
        ItemTemplate {
            id: "scroll_return".into(),
            name: "回城卷".into(),
            kind: ItemKind::Consumable,
            slot: None,
            rarity: Rarity::Common,
            price: 200,
            stackable: true,
            stats: ItemStats::default(),
        },
        ItemTemplate {
            id: "woma_horn".into(),
            name: "沃玛号角".into(),
            kind: ItemKind::Material,
            slot: None,
            rarity: Rarity::Epic,
            price: 3000,
            stackable: false,
            stats: ItemStats::default(),
        },
    ]
}
