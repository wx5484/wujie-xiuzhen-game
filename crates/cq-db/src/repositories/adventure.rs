use cq_domain::character::{initial_stats, power_from_stats, CharacterClass};
use cq_protocol::dto::{
    PlayerAdventureOfferView, PlayerAdventureOptionView, PlayerAdventureResolveView,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

use super::character::{CharacterRecord, CharacterRepository};

pub type AdventureOptionView = PlayerAdventureOptionView;
pub type AdventureOfferView = PlayerAdventureOfferView;
pub type AdventureResolveView = PlayerAdventureResolveView;

#[derive(Debug, Clone, FromRow)]
struct AdventureRow {
    id: i64,
    script_id: String,
    title: String,
    body: String,
    options: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct AdventureRepository {
    pool: PgPool,
}

impl AdventureRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn pending(&self, character_id: i64) -> Result<Option<AdventureOfferView>, sqlx::Error> {
        let row = sqlx::query_as::<_, AdventureRow>(
            r#"
            select id, script_id, title, body, options
            from character_adventures
            where character_id = $1 and status = 'pending'
            order by created_at desc
            limit 1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(adventure_view))
    }

    pub async fn maybe_trigger(
        &self,
        character: &CharacterRecord,
        zone: &str,
        room: &str,
        triggered_by: &str,
        chance_permyriad: i32,
    ) -> Result<Option<AdventureOfferView>, sqlx::Error> {
        if character.level > 499 {
            return Ok(None);
        }
        if let Some(pending) = self.pending(character.id).await? {
            return Ok(Some(pending));
        }
        let should_trigger = {
            let mut rng = thread_rng();
            chance_permyriad > 0 && rng.gen_range(0..10_000) < chance_permyriad
        };
        if !should_trigger {
            return Ok(None);
        }
        let script = {
            let mut rng = thread_rng();
            ADVENTURE_SCRIPTS.choose(&mut rng).copied().unwrap_or(ADVENTURE_SCRIPTS[0])
        };
        let options = serde_json::to_value(script.options).unwrap_or_else(|_| serde_json::json!([]));
        let row = sqlx::query_as::<_, AdventureRow>(
            r#"
            insert into character_adventures (character_id, script_id, title, body, options, triggered_by, zone, room)
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            returning id, script_id, title, body, options
            "#,
        )
        .bind(character.id)
        .bind(script.id)
        .bind(script.title)
        .bind(script.body)
        .bind(options)
        .bind(triggered_by)
        .bind(zone)
        .bind(room)
        .fetch_one(&self.pool)
        .await?;
        Ok(Some(adventure_view(row)))
    }

    pub async fn resolve(
        &self,
        character_id: i64,
        adventure_id: i64,
        option_id: &str,
    ) -> Result<AdventureResolveView, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, AdventureRow>(
            r#"
            select id, script_id, title, body, options
            from character_adventures
            where id = $1 and character_id = $2 and status = 'pending'
            for update
            "#,
        )
        .bind(adventure_id)
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        let offer = adventure_view(row);
        let selected = offer
            .options
            .iter()
            .find(|option| option.id == option_id)
            .or_else(|| offer.options.first())
            .cloned();
        let selected = selected.ok_or(sqlx::Error::RowNotFound)?;
        if selected.cost_gold > 0 {
            sqlx::query(
                r#"
                update characters
                set gold = greatest(gold - $2, 0)
                where id = $1 and deleted_at is null
                "#,
            )
            .bind(character_id)
            .bind(selected.cost_gold)
            .execute(&mut *tx)
            .await?;
        }

        let outcome = roll_outcome();
        let message = apply_outcome(&mut tx, character_id, adventure_id, &offer.title, &selected.label, outcome).await?;
        sqlx::query(
            r#"
            update character_adventures
            set status = 'resolved',
                resolved_at = now(),
                outcome = $3
            where id = $1 and character_id = $2
            "#,
        )
        .bind(adventure_id)
        .bind(character_id)
        .bind(serde_json::json!({ "message": message, "option": selected.id }))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        let character_repo = CharacterRepository::new(&self.pool);
        let character = character_repo.find(character_id).await?.ok_or(sqlx::Error::RowNotFound)?;
        let state = character_repo.state(character_id).await?;
        Ok(AdventureResolveView {
            offer,
            message,
            character: super::character::character_view(character),
            state: super::character::state_view(state),
        })
    }
}

async fn apply_outcome(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    character_id: i64,
    adventure_id: i64,
    title: &str,
    option_label: &str,
    outcome: AdventureOutcome,
) -> Result<String, sqlx::Error> {
    let message = match outcome {
        AdventureOutcome::BuffPositive => {
            let stat = random_stat();
            let pct = random_i32(50..=100);
            insert_buff(tx, character_id, adventure_id, stat, pct).await?;
            format!("奇遇【{}】选择「{}」后灵机涌动，{}提升 {}%，持续 1 小时。", title, option_label, stat_label(stat), pct)
        }
        AdventureOutcome::RewardGold => {
            let gold = random_i64(100_000..=10_000_000);
            sqlx::query("update characters set gold = gold + $2 where id = $1")
                .bind(character_id)
                .bind(gold)
                .execute(&mut **tx)
                .await?;
            format!("奇遇【{}】选择「{}」后天降横财，金币 +{}。", title, option_label, gold)
        }
        AdventureOutcome::RewardLevelUp => {
            let levels = random_i32(1..=10);
            let (next_level, class): (i32, String) = sqlx::query_as(
                r#"
                update characters
                set level = least(level + $2, 500)
                where id = $1 and deleted_at is null
                returning level, class
                "#,
            )
            .bind(character_id)
            .bind(levels)
            .fetch_one(&mut **tx)
            .await?;
            let mut stats = initial_stats(class_from_str(&class), next_level);
            stats.character_id = character_id;
            let power = power_from_stats(&stats);
            sqlx::query(
                r#"
                update characters
                set power = $2
                where id = $1
                "#,
            )
            .bind(character_id)
            .bind(power)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                r#"
                update character_stats
                set "str" = $2,
                    dex = $3,
                    "int" = $4,
                    con = $5,
                    spirit = $6,
                    max_hp = $7,
                    max_mp = $8,
                    atk = $9,
                    def = $10,
                    mag = $11,
                    mdef = $12
                where character_id = $1
                "#,
            )
            .bind(character_id)
            .bind(stats.attrs.str_)
            .bind(stats.attrs.dex)
            .bind(stats.attrs.int_)
            .bind(stats.attrs.con)
            .bind(stats.attrs.spirit)
            .bind(stats.max_hp)
            .bind(stats.max_mp)
            .bind(stats.atk)
            .bind(stats.def)
            .bind(stats.mag)
            .bind(stats.mdef)
            .execute(&mut **tx)
            .await?;
            format!("奇遇【{}】选择「{}」后顿悟飞升，等级提升 {} 级。", title, option_label, levels)
        }
        AdventureOutcome::BuffNegative => {
            let stat = random_stat();
            let pct = -random_i32(10..=50);
            insert_buff(tx, character_id, adventure_id, stat, pct).await?;
            format!("奇遇【{}】选择「{}」后气机逆转，{}降低 {}%，持续 1 小时。", title, option_label, stat_label(stat), pct.abs())
        }
        AdventureOutcome::PunishGold => {
            let gold = random_i64(1_000_000..=5_000_000);
            sqlx::query("update characters set gold = greatest(gold - $2, 0) where id = $1")
                .bind(character_id)
                .bind(gold)
                .execute(&mut **tx)
                .await?;
            format!("奇遇【{}】选择「{}」后遗失钱财，金币 -{}。", title, option_label, gold)
        }
        AdventureOutcome::PunishDeath => {
            sqlx::query(
                r#"
                update character_state
                set hp = 0,
                    zone = case
                      when zone = 'xiuzhen' then 'xiuzhen'
                      when zone = 'feisheng' then 'feisheng'
                      when zone = 'ancient_secret' then 'ancient_secret'
                      else 'fanchen'
                    end,
                    room = case
                      when zone = 'xiuzhen' then 'tianshui_city'
                      when zone = 'feisheng' then 'void_fortress'
                      when zone = 'ancient_secret' then 'taichu_camp'
                      else 'qingniu_city'
                    end
                where character_id = $1
                "#,
            )
            .bind(character_id)
            .execute(&mut **tx)
            .await?;
            sqlx::query("update afk_sessions set active = false where character_id = $1")
                .bind(character_id)
                .execute(&mut **tx)
                .await?;
            format!("奇遇【{}】选择「{}」后走火入魔，生命清零并被强制传回界域主城，挂机已中止。", title, option_label)
        }
    };
    Ok(message)
}

async fn insert_buff(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    character_id: i64,
    adventure_id: i64,
    stat: &'static str,
    pct: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        insert into character_adventure_buffs (character_id, source_adventure_id, stat, pct, expires_at)
        values ($1, $2, $3, $4, now() + interval '1 hour')
        "#,
    )
    .bind(character_id)
    .bind(adventure_id)
    .bind(stat)
    .bind(pct)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn adventure_view(row: AdventureRow) -> AdventureOfferView {
    let options = serde_json::from_value::<Vec<AdventureOptionView>>(row.options).unwrap_or_default();
    AdventureOfferView {
        id: row.id,
        script_id: row.script_id,
        title: row.title,
        body: row.body,
        options,
    }
}

#[derive(Debug, Clone, Copy)]
enum AdventureOutcome {
    BuffPositive,
    RewardGold,
    RewardLevelUp,
    BuffNegative,
    PunishGold,
    PunishDeath,
}

fn roll_outcome() -> AdventureOutcome {
    match thread_rng().gen_range(0..6) {
        0 => AdventureOutcome::BuffPositive,
        1 => AdventureOutcome::RewardGold,
        2 => AdventureOutcome::RewardLevelUp,
        3 => AdventureOutcome::BuffNegative,
        4 => AdventureOutcome::PunishGold,
        _ => AdventureOutcome::PunishDeath,
    }
}

fn random_stat() -> &'static str {
    let mut rng = thread_rng();
    ["atk", "hp", "def"][rng.gen_range(0..3)]
}

fn random_i32(range: std::ops::RangeInclusive<i32>) -> i32 {
    thread_rng().gen_range(range)
}

fn random_i64(range: std::ops::RangeInclusive<i64>) -> i64 {
    thread_rng().gen_range(range)
}

fn stat_label(stat: &str) -> &'static str {
    match stat {
        "atk" => "攻击",
        "hp" => "生命",
        "def" => "防御",
        _ => "属性",
    }
}

fn class_from_str(value: &str) -> CharacterClass {
    match value {
        "mage" => CharacterClass::Mage,
        "taoist" => CharacterClass::Taoist,
        "assassin" => CharacterClass::Assassin,
        _ => CharacterClass::Warrior,
    }
}

#[derive(Debug, Clone, Copy)]
struct AdventureScript {
    id: &'static str,
    title: &'static str,
    body: &'static str,
    options: &'static [AdventureOptionViewConst],
}

#[derive(Debug, Clone, Copy, Serialize)]
struct AdventureOptionViewConst {
    id: &'static str,
    label: &'static str,
    cost_gold: i64,
}

const ADVENTURE_SCRIPTS: &[AdventureScript] = &[
    AdventureScript { id: "miasma_array", title: "误入毒瘴阵", body: "浓绿毒瘴忽然合拢，脚下阵纹像活物一样游动。", options: &[opt("a", "闭气强冲", 0), opt("b", "服下解毒丹尝试破解", 50_000)] },
    AdventureScript { id: "ancient_tablet", title: "残碑问道", body: "一块残碑浮现古字，似乎在询问你的道心。", options: &[opt("a", "以血描摹古字", 0), opt("b", "焚香静悟", 30_000)] },
    AdventureScript { id: "wandering_merchant", title: "迷途商人", body: "背着巨大箱笼的商人拦住去路，笑容看不出真假。", options: &[opt("a", "接受神秘交易", 100_000), opt("b", "婉拒并护送一程", 0)] },
    AdventureScript { id: "thunder_pool_echo", title: "雷池回响", body: "远处雷声滚过经脉，体内灵力忽然失序。", options: &[opt("a", "借雷淬体", 0), opt("b", "立刻设阵避雷", 80_000)] },
    AdventureScript { id: "fox_moon_dream", title: "狐月入梦", body: "月光落在眉心，耳边传来天狐低语。", options: &[opt("a", "追随梦中月影", 0), opt("b", "咬破舌尖醒神", 0)] },
    AdventureScript { id: "mine_heart", title: "源石心跳", body: "一枚源石发出心跳般的震动，四周灵气被它吞吐。", options: &[opt("a", "握住源石", 0), opt("b", "以金币布置封灵阵", 120_000)] },
    AdventureScript { id: "ghost_ferry", title: "忘川渡票", body: "无人的小舟靠岸，船头放着一张写有你名字的渡票。", options: &[opt("a", "登船渡河", 0), opt("b", "撕毁渡票", 0)] },
    AdventureScript { id: "fallen_sword", title: "坠星断剑", body: "一柄断剑从天而降，剑身仍有星火流淌。", options: &[opt("a", "拔剑试锋", 0), opt("b", "以灵力温养剑身", 60_000)] },
    AdventureScript { id: "immortal_shadow", title: "谪仙残影", body: "云中落下一道残影，抬手便点向你的眉心。", options: &[opt("a", "正面承受指点", 0), opt("b", "后撤三步行礼", 0)] },
    AdventureScript { id: "abyss_whisper", title: "深渊低语", body: "黑暗中有人呼唤你的真名，声音像从心底传来。", options: &[opt("a", "回应低语", 0), opt("b", "默念清心诀", 40_000)] },
];

const fn opt(id: &'static str, label: &'static str, cost_gold: i64) -> AdventureOptionViewConst {
    AdventureOptionViewConst { id, label, cost_gold }
}
