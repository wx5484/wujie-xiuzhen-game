use cq_domain::item::Rarity;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropRoll {
    pub template_id: String,
    pub quantity: i64,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobDropKind {
    Normal,
    Elite,
    Boss,
}

pub fn roll_basic_drop(boss: bool) -> Vec<DropRoll> {
    roll_from_table(if boss { boss_loot_table() } else { basic_loot_table() }, boss, 0)
}

pub fn roll_level_drop(level: i32, boss: bool) -> Vec<DropRoll> {
    roll_level_drop_with_luck(level, boss, 0)
}

pub fn roll_level_drop_with_luck(level: i32, boss: bool, luck: i64) -> Vec<DropRoll> {
    roll_level_drop_for_source(
        level,
        if boss { MobDropKind::Boss } else { MobDropKind::Normal },
        luck,
        None,
        0,
    )
}

pub fn roll_level_drop_for_source(
    level: i32,
    kind: MobDropKind,
    luck: i64,
    source_id: Option<&str>,
    rare_quality_bonus_pct: i64,
) -> Vec<DropRoll> {
    let boss = matches!(kind, MobDropKind::Boss);
    let mut drops = roll_equipment_drops(level, kind, luck, source_id, rare_quality_bonus_pct);
    drops.extend(roll_sustain_loot(level, kind, luck));
    drops.extend(roll_skill_book(source_id, boss, luck));
    drops.extend(roll_boss_cultivation_pill(boss, luck));
    drops.extend(roll_boss_growth_materials(source_id, boss, luck));
    drops.extend(roll_yuanbao_relic(source_id, luck));
    drops
}

fn roll_from_table(templates: Vec<LootTemplate>, boss: bool, luck: i64) -> Vec<DropRoll> {
    if templates.is_empty() {
        return vec![];
    }
    let mut rng = thread_rng();
    let luck_bonus = (luck.max(0) as f64 * 0.00045).min(0.14);
    let chance = if boss { 1.0 } else { (0.28 + luck_bonus).min(0.48) };
    if rng.gen::<f64>() > chance {
        return vec![];
    }

    let weighted = templates
        .iter()
        .map(|item| {
            let bonus = rarity_rank(item.rarity) * u32::try_from((luck.max(0) / 20).min(30)).unwrap_or(0);
            (item, item.weight + bonus)
        })
        .collect::<Vec<_>>();
    let total_weight = weighted.iter().map(|(_, weight)| *weight).sum::<u32>().max(1);
    let rolls = if boss { rng.gen_range(2..=3) } else { 1 };
    (0..rolls)
        .map(|_| {
            let mut roll = rng.gen_range(0..total_weight);
            let template = weighted
                .iter()
                .find_map(|(item, weight)| {
                    if roll < *weight {
                        Some(*item)
                    } else {
                        roll -= *weight;
                        None
                    }
                })
                .unwrap_or(weighted[0].0);
            DropRoll {
                template_id: template.id.into(),
                quantity: if template.stackable { rng.gen_range(1..=3) } else { 1 },
                rarity: template.rarity,
            }
        })
        .collect()
}

fn roll_equipment_drops(
    level: i32,
    kind: MobDropKind,
    luck: i64,
    source_id: Option<&str>,
    rare_quality_bonus_pct: i64,
) -> Vec<DropRoll> {
    let _ = (kind, luck, rare_quality_bonus_pct);
    let mut rng = thread_rng();
    if rng.gen::<f64>() >= 0.50 {
        return vec![];
    }

    let max_tier = max_drop_tier_for_source(source_id).unwrap_or_else(|| max_drop_tier_for_level(level));
    let tier = roll_dynamic_tier(max_tier, &mut rng);
    vec![DropRoll {
        template_id: roll_dynamic_equipment_template(tier, &mut rng),
        quantity: 1,
        rarity: rarity_for_tier(tier),
    }]
}

fn chaos_equipment_table<R: Rng>(rng: &mut R) -> Vec<LootTemplate> {
    let roll = rng.gen_range(0..100);
    if roll < 50 {
        tier10()
    } else if roll < 80 {
        tier11()
    } else {
        tier12()
    }
}

fn max_drop_tier_for_level(level: i32) -> i32 {
    match level.max(1) {
        1..=20 => 1,
        21..=40 => 2,
        41..=80 => 3,
        81..=120 => 4,
        121..=160 => 5,
        161..=200 => 6,
        201..=220 => 7,
        221..=240 => 8,
        241..=280 => 9,
        281..=300 => 10,
        301..=340 => 11,
        341..=380 => 12,
        381..=420 => 13,
        421..=460 => 14,
        461..=480 => 15,
        481..=499 => 16,
        _ => 17,
    }
}

fn max_drop_tier_for_source(source_id: Option<&str>) -> Option<i32> {
    let source_id = source_id?;
    let mut parts = source_id.split(':');
    let zone = parts.next()?;
    let room = parts.next()?;
    let tier = match (zone, room) {
        ("fanchen", "bamboo_outer" | "bamboo_fork" | "dense_river" | "bamboo_depths" | "mine_entrance" | "abandoned_mine" | "collapsed_passage" | "foreman_room") => 1,
        ("fanchen", "wilderness_edge" | "bone_highland" | "wilderness_depths" | "wilderness_altar" | "qingniu_plain" | "tianshui_ancient_road") => 2,
        ("xiuzhen", "poison_marsh" | "flower_secret" | "ape_ravine" | "wanyao_cave") => 3,
        ("xiuzhen", "tower_floor_1" | "tower_floor_2" | "tower_floor_3" | "tower_top_seal" | "tower_undercurrent") => 4,
        ("xiuzhen", "ice_forest" | "winter_canyon" | "dragonbone_icefield" | "ice_throne" | "icebreaking_route") => 5,
        ("feisheng", "wangchuan_bank" | "lost_ferry" | "nether_water_palace" | "judge_hall") => 6,
        ("feisheng", "moon_maze") => 7,
        ("feisheng", "silver_fox_square" | "moon_worship_platform" | "skyfox_inner_palace") => 8,
        ("feisheng", "abyss_shallow" | "abyss_cave") => 9,
        ("feisheng", "chaos_storm" | "abyss_beast_lair" | "ascension_thunder_pool") => 10,
        ("ancient_secret", "mining_outer" | "source_crystal_cave" | "source_throne") => 11,
        ("ancient_secret", "miasma_forest" | "devouring_snake_marsh" | "giant_ape_domain") => 12,
        ("ancient_secret", "ruined_gate" | "immortal_steps" | "lingxiao_ruins") => 13,
        ("ancient_secret", "ash_plain" | "fire_nest" | "nirvana_core") => 14,
        ("ancient_secret", "broken_stars") => 15,
        ("ancient_secret", "beast_meteor" | "heavenly_dao_altar") => 16,
        ("ancient_secret", "stargazer_observatory") => 17,
        _ => return None,
    };
    Some(tier)
}

fn roll_dynamic_tier<R: Rng>(max_tier: i32, rng: &mut R) -> i32 {
    let max_tier = max_tier.clamp(1, 17);
    let roll = rng.gen_range(0..100);
    let tier = match roll {
        0..=49 => max_tier - 2,
        50..=79 => max_tier - 1,
        80..=98 => max_tier,
        _ => max_tier + 1,
    };
    tier.clamp(1, 17)
}

fn roll_dynamic_equipment_template<R: Rng>(tier: i32, rng: &mut R) -> String {
    let slot = match rng.gen_range(0..100) {
        0..=49 => random_item(&["chest", "head", "feet"], rng),
        50..=98 => random_item(&["neck", "bracelet", "ring", "waist"], rng),
        _ => "weapon",
    };
    format!("t{tier:02}_{slot}", tier = tier.clamp(1, 17), slot = slot)
}

fn random_item<'a, R: Rng>(items: &'a [&'a str], rng: &mut R) -> &'a str {
    items[rng.gen_range(0..items.len())]
}

fn rarity_for_tier(tier: i32) -> Rarity {
    match tier.clamp(1, 17) {
        1 => Rarity::Common,
        2 => Rarity::Uncommon,
        3 | 4 => Rarity::Rare,
        5..=7 => Rarity::Epic,
        8..=11 => Rarity::Legendary,
        12..=14 => Rarity::Mythic,
        15 | 16 => Rarity::Supreme,
        _ => Rarity::Ultimate,
    }
}

fn roll_sustain_loot(level: i32, kind: MobDropKind, luck: i64) -> Vec<DropRoll> {
    roll_from_table(sustain_loot(level), matches!(kind, MobDropKind::Boss), luck)
}

fn roll_quality<R: Rng>(level: i32, kind: MobDropKind, rare_quality_bonus_pct: i64, rng: &mut R) -> Rarity {
    if matches!(kind, MobDropKind::Boss) && level >= 100 && rng.gen_range(0..100) == 0 {
        return Rarity::Legendary;
    }

    let bonus = u32::try_from(rare_quality_bonus_pct.clamp(0, 30)).unwrap_or(0);
    let mut weights = match kind {
        MobDropKind::Normal => vec![
            (Rarity::Common, 50),
            (Rarity::Uncommon, 40),
            (Rarity::Rare, 10 + bonus),
        ],
        MobDropKind::Elite => vec![
            (Rarity::Common, 20),
            (Rarity::Uncommon, 50),
            (Rarity::Rare, 20 + bonus),
            (Rarity::Epic, 10 + bonus / 2),
        ],
        MobDropKind::Boss => vec![
            (Rarity::Common, 12),
            (Rarity::Uncommon, 43),
            (Rarity::Rare, 30 + bonus),
            (Rarity::Epic, 15 + bonus / 2),
        ],
    };
    if !matches!(kind, MobDropKind::Normal) && level < 18 {
        weights.retain(|(rarity, _)| !matches!(rarity, Rarity::Epic));
    }
    choose_rarity(&weights, rng)
}

fn choose_rarity<R: Rng>(weights: &[(Rarity, u32)], rng: &mut R) -> Rarity {
    let total = weights.iter().map(|(_, weight)| *weight).sum::<u32>().max(1);
    let mut roll = rng.gen_range(0..total);
    for (rarity, weight) in weights {
        if roll < *weight {
            return *rarity;
        }
        roll -= *weight;
    }
    weights.first().map(|(rarity, _)| *rarity).unwrap_or(Rarity::Common)
}

fn candidates_for_rarity(templates: &[LootTemplate], rarity: Rarity) -> Vec<LootTemplate> {
    templates
        .iter()
        .copied()
        .filter(|template| template.rarity == rarity)
        .collect()
}

fn choose_weighted<R: Rng>(templates: &[LootTemplate], luck: i64, rng: &mut R) -> Option<LootTemplate> {
    if templates.is_empty() {
        return None;
    }
    let weighted = templates
        .iter()
        .map(|item| {
            let bonus = rarity_rank(item.rarity) * u32::try_from((luck.max(0) / 20).min(30)).unwrap_or(0);
            (*item, item.weight + bonus)
        })
        .collect::<Vec<_>>();
    let total_weight = weighted.iter().map(|(_, weight)| *weight).sum::<u32>().max(1);
    let mut roll = rng.gen_range(0..total_weight);
    weighted
        .iter()
        .find_map(|(item, weight)| {
            if roll < *weight {
                Some(*item)
            } else {
                roll -= *weight;
                None
            }
        })
        .or_else(|| weighted.first().map(|(item, _)| *item))
}

fn roll_skill_book(source_id: Option<&str>, boss: bool, luck: i64) -> Vec<DropRoll> {
    if !boss {
        return vec![];
    }
    let books = skill_book_table_for_source(source_id);
    if books.is_empty() {
        return vec![];
    }
    let mut rng = thread_rng();
    let luck_bonus = (luck.max(0) as f64 * 0.00003).min(0.03);
    let chance = if source_has_id(source_id, "boss_wraith_aggregate") {
        0.22 + luck_bonus
    } else if source_has_id(source_id, "boss_yanluo_judge") {
        0.0001
    } else {
        0.12 + luck_bonus
    };
    if rng.gen::<f64>() > chance {
        return vec![];
    }
    let total_weight = books.iter().map(|item| item.weight).sum::<u32>().max(1);
    let mut roll = rng.gen_range(0..total_weight);
    let book = books
        .iter()
        .find(|item| {
            if roll < item.weight {
                true
            } else {
                roll -= item.weight;
                false
            }
        })
        .unwrap_or(&books[0]);
    vec![drop(book.id, 1, book.rarity)]
}

fn roll_boss_cultivation_pill(boss: bool, luck: i64) -> Vec<DropRoll> {
    if !boss {
        return vec![];
    }
    let luck_bonus = (luck.max(0) as f64 * 0.00001).min(0.008);
    if thread_rng().gen::<f64>() <= 0.006 + luck_bonus {
        vec![drop("pill_cultivate", 1, Rarity::Rare)]
    } else {
        vec![]
    }
}

fn roll_boss_growth_materials(source_id: Option<&str>, boss: bool, luck: i64) -> Vec<DropRoll> {
    if !boss {
        return vec![];
    }
    let luck_bonus = (luck.max(0) as f64 * 0.000003).min(0.003);
    let mut rng = thread_rng();
    let mut drops = Vec::new();
    if rng.gen::<f64>() <= 0.0025 + luck_bonus {
        drops.push(drop("pet_food", 1, Rarity::Uncommon));
    }
    if rng.gen::<f64>() <= 0.002 + luck_bonus {
        drops.push(drop("treasure_shard", 1, Rarity::Rare));
    }
    if rng.gen::<f64>() <= 0.0015 + luck_bonus {
        drops.push(drop("cultivation_pill", 1, Rarity::Rare));
    }
    if rng.gen::<f64>() <= 0.001 + luck_bonus {
        drops.push(drop("guild_merit_token", 1, Rarity::Uncommon));
    }
    if is_zuma_or_higher_boss(source_id) {
        let page_bonus = (luck.max(0) as f64 * 0.00002).min(0.02);
        if rng.gen::<f64>() <= 0.16 + page_bonus {
            drops.push(drop("skill_page", rng.gen_range(1..=3), Rarity::Rare));
        }
    }
    drops
}

fn roll_yuanbao_relic(source_id: Option<&str>, luck: i64) -> Vec<DropRoll> {
    let Some(source_id) = source_id else {
        return vec![];
    };
    let _ = luck;
    let mut rng = thread_rng();
    if source_has_id(Some(source_id), "boss_boundary_stonemaw")
        && rng.gen::<f64>() <= 0.01
    {
        return vec![drop("yuanbao_glory", 1, Rarity::Legendary)];
    }
    if source_has_id(Some(source_id), "boss_icefield_overlord") && rng.gen::<f64>() <= 0.01 {
        return vec![drop("yuanbao_legacy", 1, Rarity::Legendary)];
    }
    vec![]
}

fn drop(template_id: &str, quantity: i64, rarity: Rarity) -> DropRoll {
    DropRoll { template_id: template_id.into(), quantity, rarity }
}

#[derive(Debug, Clone, Copy)]
struct LootTemplate {
    id: &'static str,
    rarity: Rarity,
    stackable: bool,
    weight: u32,
}

fn loot(id: &'static str, rarity: Rarity, weight: u32) -> LootTemplate {
    LootTemplate { id, rarity, stackable: false, weight }
}

fn stack(id: &'static str, rarity: Rarity, weight: u32) -> LootTemplate {
    LootTemplate { id, rarity, stackable: true, weight }
}

fn basic_loot_table() -> Vec<LootTemplate> {
    vec![
        loot("sword_wood", Rarity::Common, 18),
        loot("armor_cloth", Rarity::Common, 18),
        loot("ring_copper", Rarity::Uncommon, 10),
        loot("blade_green_t1", Rarity::Uncommon, 8),
        loot("armor_jade_t1", Rarity::Uncommon, 8),
        loot("helm_bronze_t1", Rarity::Uncommon, 6),
        loot("boots_deerskin_t1", Rarity::Uncommon, 6),
        loot("bracelet_guard_t1", Rarity::Uncommon, 6),
    ]
}

fn boss_loot_table() -> Vec<LootTemplate> {
    let mut items = Vec::new();
    items.extend(tier5());
    items.extend(tier6());
    items.extend(tier7());
    items.extend(tier8());
    items
}

fn level_loot_table(level: i32, boss: bool) -> Vec<LootTemplate> {
    let mut items = sustain_loot(level);
    items.extend(normal_equipment_loot(level));
    if boss {
        if level >= 70 {
            items.push(stack("stone_hongmeng", Rarity::Legendary, 1));
        }
        items.extend(boss_equipment_loot(level));
    }
    items
}

fn level_equipment_table(level: i32, kind: MobDropKind, source_id: Option<&str>) -> Vec<LootTemplate> {
    let mut items = normal_equipment_loot(level);
    if matches!(kind, MobDropKind::Elite | MobDropKind::Boss) {
        items.extend(elite_equipment_loot(level));
    }
    if matches!(kind, MobDropKind::Boss) {
        items.extend(boss_equipment_loot(level));
        items.extend(exclusive_boss_loot(source_id));
    }
    items
}

fn fallback_equipment_table(level: i32) -> Vec<LootTemplate> {
    let mut items = tier1();
    items.extend(tier2());
    items.extend(elite_equipment_loot(level));
    items.extend(boss_equipment_loot(level));
    items
}

fn sustain_loot(_level: i32) -> Vec<LootTemplate> {
    vec![stack("stone_refine", Rarity::Uncommon, 4)]
}

fn normal_equipment_loot(level: i32) -> Vec<LootTemplate> {
    let mut items = Vec::new();
    match level {
        ..=10 => items.extend(tier1()),
        11..=25 => {
            items.extend(scaled_loot(tier1(), 2));
            items.extend(tier2());
        }
        26..=45 => {
            items.extend(scaled_loot(tier1(), 4));
            items.extend(tier2());
            items.extend(scaled_loot(tier3(), 5));
        }
        46..=70 => {
            items.extend(scaled_loot(tier1(), 5));
            items.extend(scaled_loot(tier2(), 2));
            items.extend(scaled_loot(tier3(), 3));
            items.extend(scaled_loot(non_legendary(tier4()), 6));
        }
        71..=95 => {
            items.extend(scaled_loot(tier1(), 6));
            items.extend(scaled_loot(tier2(), 3));
            items.extend(scaled_loot(tier3(), 2));
            items.extend(scaled_loot(non_legendary(tier4()), 4));
            items.extend(scaled_loot(non_legendary(tier5()), 7));
        }
        96..=125 => {
            items.extend(scaled_loot(tier1(), 7));
            items.extend(scaled_loot(tier2(), 4));
            items.extend(scaled_loot(non_legendary(tier4()), 3));
            items.extend(scaled_loot(non_legendary(tier5()), 5));
            items.extend(scaled_loot(non_legendary(tier6()), 8));
        }
        126..=155 => {
            items.extend(scaled_loot(tier1(), 8));
            items.extend(scaled_loot(tier2(), 5));
            items.extend(scaled_loot(non_legendary(tier5()), 4));
            items.extend(scaled_loot(non_legendary(tier6()), 6));
            items.extend(scaled_loot(non_legendary(tier7()), 9));
        }
        156..=180 => {
            items.extend(scaled_loot(tier1(), 9));
            items.extend(scaled_loot(tier2(), 6));
            items.extend(scaled_loot(non_legendary(tier6()), 5));
            items.extend(scaled_loot(non_legendary(tier7()), 7));
            items.extend(scaled_loot(non_legendary(tier8()), 10));
        }
        181..=195 => {
            items.extend(scaled_loot(non_legendary(tier8()), 8));
            items.extend(scaled_loot(tier9(), 10));
            items.extend(scaled_loot(tier10(), 12));
        }
        _ => {
            items.extend(scaled_loot(tier10(), 10));
            items.extend(scaled_loot(tier11(), 12));
            items.extend(scaled_loot(tier12(), 16));
        }
    }
    items
}

fn elite_equipment_loot(level: i32) -> Vec<LootTemplate> {
    match level {
        ..=20 => tier3(),
        21..=45 => {
            let mut items = tier3();
            items.extend(non_legendary(tier4()));
            items
        }
        46..=70 => {
            let mut items = non_legendary(tier4());
            items.extend(non_legendary(tier5()));
            items
        }
        71..=95 => {
            let mut items = non_legendary(tier5());
            items.extend(non_legendary(tier6()));
            items
        }
        96..=125 => {
            let mut items = non_legendary(tier6());
            items.extend(non_legendary(tier7()));
            items
        }
        126..=155 => {
            let mut items = non_legendary(tier7());
            items.extend(non_legendary(tier8()));
            items
        }
        156..=180 => {
            let mut items = non_legendary(tier8());
            items.extend(non_legendary(tier9()));
            items
        }
        181..=195 => {
            let mut items = tier9();
            items.extend(tier10());
            items.extend(tier11());
            items
        }
        _ => {
            let mut items = tier10();
            items.extend(tier11());
            items.extend(tier12());
            items
        }
    }
}

fn boss_equipment_loot(level: i32) -> Vec<LootTemplate> {
    match level {
        ..=20 => tier3(),
        21..=35 => tier4(),
        36..=50 => tier5(),
        51..=70 => tier6(),
        71..=90 => tier7(),
        91..=110 => tier8(),
        111..=150 => tier9(),
        151..=180 => tier10(),
        181..=195 => tier11(),
        _ => tier12(),
    }
}

fn exclusive_boss_loot(source_id: Option<&str>) -> Vec<LootTemplate> {
    let source_id = source_id.unwrap_or_default();
    if source_id.contains("woma_lord") {
        return woma_set();
    }
    if source_id.contains("zuma_lord") {
        return zuma_set();
    }
    if source_id.contains("bull_king") || source_id.contains("mirage_bull") {
        return mirage_set();
    }
    if source_id.contains("redmoon") || source_id.contains("twin_head") {
        return redmoon_set();
    }
    if source_id.contains("molong") {
        return molong_set();
    }
    if source_id.contains("huyue") {
        return tier11();
    }
    if source_id.contains("chaos") {
        return tier12();
    }
    vec![]
}

fn is_chaos_source(source_id: Option<&str>) -> bool {
    let source_id = source_id.unwrap_or_default();
    source_id.contains("chaos_abyss") || source_id.contains("chaos")
}

fn non_legendary(items: Vec<LootTemplate>) -> Vec<LootTemplate> {
    items
        .into_iter()
        .filter(|item| !matches!(item.rarity, Rarity::Legendary | Rarity::Mythic | Rarity::Supreme | Rarity::Ultimate))
        .collect()
}

fn scaled_loot(mut items: Vec<LootTemplate>, divisor: u32) -> Vec<LootTemplate> {
    let divisor = divisor.max(1);
    for item in &mut items {
        item.weight = (item.weight / divisor).max(1);
    }
    items
}

fn tier1() -> Vec<LootTemplate> {
    vec![
        loot("sword_wood", Rarity::Common, 22),
        loot("armor_cloth", Rarity::Common, 22),
        loot("ring_copper", Rarity::Uncommon, 14),
        loot("blade_green_t1", Rarity::Uncommon, 12),
        loot("armor_jade_t1", Rarity::Uncommon, 12),
        loot("helm_bronze_t1", Rarity::Uncommon, 10),
        loot("boots_deerskin_t1", Rarity::Uncommon, 10),
        loot("bracelet_guard_t1", Rarity::Uncommon, 10),
    ]
}

fn tier2() -> Vec<LootTemplate> {
    vec![
        loot("blade_darkiron_t2", Rarity::Rare, 18),
        loot("armor_cloud_t2", Rarity::Rare, 18),
        loot("ring_sea_t2", Rarity::Rare, 14),
        loot("neck_jade_t2", Rarity::Rare, 12),
        loot("bracelet_tiger_t2", Rarity::Rare, 12),
        loot("blade_serpent_t2", Rarity::Rare, 10),
        loot("belt_snake_t2", Rarity::Rare, 10),
        loot("boots_cloud_t2", Rarity::Rare, 10),
    ]
}

fn tier3() -> Vec<LootTemplate> {
    vec![
        loot("blade_flame_t3", Rarity::Epic, 18),
        loot("armor_star_t3", Rarity::Epic, 18),
        loot("ring_thunder_t3", Rarity::Epic, 14),
        loot("armor_woma_t3", Rarity::Epic, 12),
        loot("helm_woma_t3", Rarity::Epic, 12),
        loot("neck_woma_t3", Rarity::Epic, 12),
        loot("bracelet_woma_t3", Rarity::Epic, 10),
        loot("belt_woma_t3", Rarity::Epic, 10),
        loot("boots_woma_t3", Rarity::Epic, 10),
    ]
}

fn tier4() -> Vec<LootTemplate> {
    vec![
        loot("blade_dragon_t4", Rarity::Legendary, 8),
        loot("armor_phoenix_t4", Rarity::Legendary, 8),
        loot("ring_sun_t4", Rarity::Legendary, 6),
        loot("blade_purgatory_t4", Rarity::Epic, 12),
        loot("armor_dragon_t4", Rarity::Epic, 12),
        loot("ring_zuma_t4", Rarity::Epic, 12),
        loot("belt_dragon_t4", Rarity::Legendary, 8),
        loot("helm_zuma_t4", Rarity::Epic, 10),
        loot("bracelet_zuma_t4", Rarity::Epic, 10),
        loot("armor_zuma_t4", Rarity::Epic, 10),
        loot("neck_zuma_t4", Rarity::Epic, 10),
        loot("belt_zuma_t4", Rarity::Epic, 10),
        loot("boots_zuma_t4", Rarity::Epic, 10),
    ]
}

fn tier5() -> Vec<LootTemplate> {
    vec![
        loot("blade_cangyue_t5", Rarity::Epic, 14),
        loot("armor_bull_t5", Rarity::Epic, 14),
        loot("neck_nether_t5", Rarity::Epic, 12),
        loot("boss_relic_blade", Rarity::Legendary, 8),
        loot("boss_relic_armor", Rarity::Legendary, 8),
        loot("ring_bull_t5", Rarity::Epic, 10),
        loot("bracelet_nether_t5", Rarity::Epic, 10),
        loot("boots_cangyue_t5", Rarity::Epic, 10),
        loot("armor_mirage_t5", Rarity::Epic, 10),
        loot("helm_mirage_t5", Rarity::Epic, 10),
        loot("neck_mirage_t5", Rarity::Epic, 10),
        loot("bracelet_mirage_t5", Rarity::Epic, 10),
        loot("belt_mirage_t5", Rarity::Epic, 10),
        loot("boots_mirage_t5", Rarity::Epic, 10),
    ]
}

fn tier6() -> Vec<LootTemplate> {
    vec![
        loot("blade_soul_t6", Rarity::Epic, 12),
        loot("armor_soul_t6", Rarity::Epic, 12),
        loot("helm_soul_t6", Rarity::Epic, 10),
        loot("ring_soul_t6", Rarity::Legendary, 10),
        loot("neck_soul_t6", Rarity::Epic, 10),
        loot("bracelet_soul_t6", Rarity::Epic, 10),
    ]
}

fn tier7() -> Vec<LootTemplate> {
    vec![
        loot("blade_starfall_t7", Rarity::Legendary, 10),
        loot("armor_starfall_t7", Rarity::Epic, 10),
        loot("helm_starfall_t7", Rarity::Epic, 9),
        loot("ring_starfall_t7", Rarity::Legendary, 9),
        loot("neck_starfall_t7", Rarity::Epic, 9),
        loot("bracelet_starfall_t7", Rarity::Epic, 9),
    ]
}

fn tier8() -> Vec<LootTemplate> {
    vec![
        loot("blade_void_t8", Rarity::Legendary, 8),
        loot("armor_void_t8", Rarity::Legendary, 8),
        loot("helm_void_t8", Rarity::Epic, 7),
        loot("ring_void_t8", Rarity::Legendary, 7),
        loot("neck_void_t8", Rarity::Epic, 7),
        loot("bracelet_void_t8", Rarity::Epic, 7),
        loot("blade_redmoon_t8", Rarity::Legendary, 4),
        loot("armor_redmoon_t8", Rarity::Legendary, 4),
        loot("helm_redmoon_t8", Rarity::Legendary, 4),
        loot("neck_redmoon_t8", Rarity::Legendary, 4),
        loot("bracelet_redmoon_t8", Rarity::Legendary, 4),
        loot("ring_redmoon_t8", Rarity::Legendary, 4),
        loot("boots_redmoon_t8", Rarity::Legendary, 4),
    ]
}

fn tier9() -> Vec<LootTemplate> {
    vec![
        loot("blade_god_t9", Rarity::Legendary, 6),
        loot("armor_god_t9", Rarity::Legendary, 6),
        loot("helm_god_t9", Rarity::Epic, 5),
        loot("ring_god_t9", Rarity::Legendary, 5),
        loot("neck_god_t9", Rarity::Legendary, 5),
        loot("bracelet_god_t9", Rarity::Epic, 5),
    ]
}

fn tier10() -> Vec<LootTemplate> {
    vec![
        loot("blade_immortal_t10", Rarity::Legendary, 4),
        loot("armor_immortal_t10", Rarity::Legendary, 4),
        loot("helm_immortal_t10", Rarity::Epic, 3),
        loot("ring_immortal_t10", Rarity::Legendary, 3),
        loot("neck_immortal_t10", Rarity::Legendary, 3),
        loot("bracelet_immortal_t10", Rarity::Legendary, 3),
        loot("blade_molong_t10", Rarity::Legendary, 2),
        loot("armor_molong_t10", Rarity::Legendary, 2),
        loot("helm_molong_t10", Rarity::Legendary, 2),
        loot("neck_molong_t10", Rarity::Legendary, 2),
        loot("bracelet_molong_t10", Rarity::Legendary, 2),
        loot("ring_molong_t10", Rarity::Legendary, 2),
        loot("boots_molong_t10", Rarity::Legendary, 2),
    ]
}

fn tier11() -> Vec<LootTemplate> {
    vec![
        loot("blade_huyue_t11", Rarity::Legendary, 5),
        loot("armor_huyue_t11", Rarity::Legendary, 5),
        loot("helm_huyue_t11", Rarity::Legendary, 4),
        loot("neck_huyue_t11", Rarity::Legendary, 4),
        loot("bracelet_huyue_t11", Rarity::Legendary, 4),
        loot("ring_huyue_t11", Rarity::Legendary, 4),
        loot("belt_huyue_t11", Rarity::Legendary, 4),
        loot("boots_huyue_t11", Rarity::Legendary, 4),
    ]
}

fn tier12() -> Vec<LootTemplate> {
    vec![
        loot("blade_chaos_t12", Rarity::Mythic, 4),
        loot("armor_chaos_t12", Rarity::Mythic, 4),
        loot("helm_chaos_t12", Rarity::Mythic, 3),
        loot("neck_chaos_t12", Rarity::Mythic, 3),
        loot("bracelet_chaos_t12", Rarity::Mythic, 3),
        loot("ring_chaos_t12", Rarity::Mythic, 3),
        loot("belt_chaos_t12", Rarity::Mythic, 3),
        loot("boots_chaos_t12", Rarity::Mythic, 3),
    ]
}

fn woma_set() -> Vec<LootTemplate> {
    vec![
        loot("armor_woma_t3", Rarity::Epic, 16),
        loot("helm_woma_t3", Rarity::Epic, 16),
        loot("neck_woma_t3", Rarity::Epic, 16),
        loot("bracelet_woma_t3", Rarity::Epic, 16),
        loot("belt_woma_t3", Rarity::Epic, 16),
        loot("boots_woma_t3", Rarity::Epic, 16),
    ]
}

fn zuma_set() -> Vec<LootTemplate> {
    vec![
        loot("armor_zuma_t4", Rarity::Epic, 14),
        loot("helm_zuma_t4", Rarity::Epic, 14),
        loot("neck_zuma_t4", Rarity::Epic, 14),
        loot("bracelet_zuma_t4", Rarity::Epic, 14),
        loot("ring_zuma_t4", Rarity::Epic, 14),
        loot("belt_zuma_t4", Rarity::Epic, 14),
        loot("boots_zuma_t4", Rarity::Epic, 14),
    ]
}

fn mirage_set() -> Vec<LootTemplate> {
    vec![
        loot("armor_mirage_t5", Rarity::Epic, 12),
        loot("helm_mirage_t5", Rarity::Epic, 12),
        loot("neck_mirage_t5", Rarity::Epic, 12),
        loot("bracelet_mirage_t5", Rarity::Epic, 12),
        loot("belt_mirage_t5", Rarity::Epic, 12),
        loot("boots_mirage_t5", Rarity::Epic, 12),
    ]
}

fn redmoon_set() -> Vec<LootTemplate> {
    vec![
        loot("blade_redmoon_t8", Rarity::Legendary, 8),
        loot("armor_redmoon_t8", Rarity::Legendary, 8),
        loot("helm_redmoon_t8", Rarity::Legendary, 8),
        loot("neck_redmoon_t8", Rarity::Legendary, 8),
        loot("bracelet_redmoon_t8", Rarity::Legendary, 8),
        loot("ring_redmoon_t8", Rarity::Legendary, 8),
        loot("boots_redmoon_t8", Rarity::Legendary, 8),
    ]
}

fn molong_set() -> Vec<LootTemplate> {
    vec![
        loot("blade_molong_t10", Rarity::Legendary, 6),
        loot("armor_molong_t10", Rarity::Legendary, 6),
        loot("helm_molong_t10", Rarity::Legendary, 6),
        loot("neck_molong_t10", Rarity::Legendary, 6),
        loot("bracelet_molong_t10", Rarity::Legendary, 6),
        loot("ring_molong_t10", Rarity::Legendary, 6),
        loot("boots_molong_t10", Rarity::Legendary, 6),
    ]
}

fn skill_book_table_for_source(source_id: Option<&str>) -> Vec<LootTemplate> {
    if source_has_id(source_id, "boss_wraith_aggregate") {
        return vec![
            loot("book_sword_wanfa", Rarity::Epic, 10),
            loot("book_sword_dayan", Rarity::Epic, 10),
            loot("book_spell_xuan_light", Rarity::Epic, 10),
            loot("book_spell_resonance", Rarity::Epic, 10),
            loot("book_soul_karma_fire", Rarity::Epic, 10),
            loot("book_soul_reincarnation", Rarity::Epic, 10),
        ];
    }
    if source_has_id(source_id, "boss_yanluo_judge") {
        return vec![
            loot("book_sword_clear_heart", Rarity::Legendary, 10),
            loot("book_sword_zhuxian", Rarity::Legendary, 8),
            loot("book_spell_chaos_orb", Rarity::Legendary, 10),
            loot("book_spell_thunder_array", Rarity::Legendary, 8),
            loot("book_soul_all_unity", Rarity::Legendary, 10),
            loot("book_soul_yanluo_prison", Rarity::Legendary, 8),
        ];
    }
    vec![]
}

fn is_zuma_or_higher_boss(source_id: Option<&str>) -> bool {
    [
        "zuma_lord",
        "nether_lord",
        "bull_king",
        "rainbow_lord",
        "redmoon_demon",
        "molong_blood_demon",
        "molong_lord",
        "chaos_overlord",
        "boss_wanyao_lord",
        "boss_wraith_aggregate",
        "boss_icefield_overlord",
        "boss_yanluo_judge",
        "boss_skyfox_ancestor",
        "boss_chaos_abyss_beast",
        "boss_source_golem_king",
        "boss_primordial_giant_ape",
        "boss_fallen_exiled_immortal",
        "boss_phoenix_avatar",
        "boss_heavenly_dao_phantom",
        "world_boss_eternal_abyss_demon",
    ]
    .iter()
    .any(|id| source_has_id(source_id, id))
}

fn source_has_id(source_id: Option<&str>, expected: &str) -> bool {
    let source_id = source_id.unwrap_or_default();
    source_id == expected || source_id.split(':').any(|part| part == expected) || source_id.contains(expected)
}

fn rarity_rank(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 4,
        Rarity::Legendary => 7,
        Rarity::Mythic => 10,
        Rarity::Supreme => 7,
        Rarity::Ultimate => 7,
    }
}
