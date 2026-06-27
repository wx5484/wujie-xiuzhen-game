use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterClass {
    Warrior,
    Mage,
    Taoist,
    Assassin,
}

impl CharacterClass {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Warrior => "剑修",
            Self::Mage => "法修",
            Self::Taoist => "魂修",
            Self::Assassin => "刺客",
        }
    }

    pub fn base_attributes(self) -> Attributes {
        match self {
            Self::Warrior => Attributes { str_: 14, dex: 8, int_: 3, con: 11, spirit: 4 },
            Self::Mage => Attributes { str_: 4, dex: 6, int_: 16, con: 6, spirit: 12 },
            Self::Taoist => Attributes { str_: 8, dex: 8, int_: 10, con: 10, spirit: 14 },
            Self::Assassin => Attributes { str_: 9, dex: 12, int_: 4, con: 7, spirit: 5 },
        }
    }

    pub fn growth(self) -> Growth {
        match self {
            Self::Warrior => Growth { hp: 20, mp: 4 },
            Self::Mage => Growth { hp: 8, mp: 18 },
            Self::Taoist => Growth { hp: 15, mp: 12 },
            Self::Assassin => Growth { hp: 14, mp: 6 },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Attributes {
    #[serde(rename = "str")]
    pub str_: i64,
    pub dex: i64,
    #[serde(rename = "int")]
    pub int_: i64,
    pub con: i64,
    pub spirit: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Growth {
    pub hp: i64,
    pub mp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: EntityId,
    pub account_id: EntityId,
    pub name: String,
    pub class: CharacterClass,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub power: i64,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub character_id: EntityId,
    pub attrs: Attributes,
    pub max_hp: i64,
    pub max_mp: i64,
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub character_id: EntityId,
    pub zone: String,
    pub room: String,
    pub hp: i64,
    pub mp: i64,
    pub online: bool,
    pub updated_at: OffsetDateTime,
}

pub fn initial_stats(class: CharacterClass, level: i32) -> CharacterStats {
    let attrs = class.base_attributes();
    let growth = class.growth();
    let level_bonus = i64::from(level.saturating_sub(1));
    let max_hp = 80 + attrs.con * 8 + growth.hp * level_bonus;
    let max_mp = 30 + attrs.int_ * 3 + attrs.spirit * 4 + growth.mp * level_bonus;
    CharacterStats {
        character_id: 0,
        attrs,
        max_hp,
        max_mp,
        atk: attrs.str_ * 2 + level_bonus,
        def: attrs.con + level_bonus / 2,
        mag: attrs.int_ * 2 + level_bonus,
        mdef: attrs.spirit + level_bonus / 2,
    }
}

pub fn exp_for_level(level: i32) -> i64 {
    let target_level = i128::from(level.max(1));
    if target_level <= 1 {
        return 0;
    }
    let n = target_level - 1;
    let total = 2_344_i128 * n * (n + 1) * (2 * n + 1) / 6;
    total.min(i128::from(i64::MAX)) as i64
}

pub fn power_from_stats(stats: &CharacterStats) -> i64 {
    stats.max_hp / 5
        + stats.max_mp / 8
        + stats.atk * 6
        + stats.def * 4
        + stats.mag * 6
        + stats.mdef * 4
        + stats.attrs.dex * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warrior_has_more_hp_than_mage() {
        let warrior = initial_stats(CharacterClass::Warrior, 10);
        let mage = initial_stats(CharacterClass::Mage, 10);
        assert!(warrior.max_hp > mage.max_hp);
        assert!(mage.max_mp > warrior.max_mp);
    }
}
