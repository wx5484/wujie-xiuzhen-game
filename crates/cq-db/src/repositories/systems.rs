use cq_protocol::dto::{
    PlayerCultivationView, PlayerPetView, PlayerSabakView, PlayerSystemUnlockView,
    PlayerSystemUnlocksView, PlayerSystemsView, PlayerTreasureView, PlayerVipSettingsView,
    PlayerVipView, PlayerWanxiangBodyView, RechargeCardResult, SystemActionResult,
    VipPotionSettingsRequest,
};
use rand::{thread_rng, Rng};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::character::{character_view, CharacterRecord};

const PET_MAX_LEVEL: i32 = 200;
const TREASURE_MAX_LEVEL: i32 = 200;
const TREASURE_MAX_STAGE: i32 = 20;
const CULTIVATION_MAX_LAYER: i32 = 81;
const GROWTH_MATERIAL_TOTAL: i64 = 10_000;
const WANXIANG_MAX_LEVEL: i32 = 1000;
const WANXIANG_FULL_ATK: i64 = 20_000;
const WANXIANG_FULL_MAG: i64 = 20_000;
const WANXIANG_FULL_HP: i64 = 500_000;
const WANXIANG_FULL_MP: i64 = 500_000;
const WANXIANG_FULL_DEF: i64 = 10_000;
const WANXIANG_FULL_MDEF: i64 = 10_000;
const WANXIANG_FULL_LIFE_STEAL_PCT: i64 = 10;
const WANXIANG_FULL_MANA_STEAL_PCT: i64 = 10;
const WANXIANG_FULL_DAMAGE_REDUCE_PCT: i64 = 60;

#[derive(Debug, Clone)]
pub struct SystemsRepository {
    pool: PgPool,
}

#[derive(Debug, Error)]
pub enum SystemsActionError {
    #[error("not found")]
    NotFound,
    #[error("not enough gold")]
    NotEnoughGold,
    #[error("not enough material")]
    NotEnoughMaterial,
    #[error("max level reached")]
    MaxLevel,
    #[error("{0}")]
    Locked(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct SystemBonus {
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
    pub hp: i64,
    pub mp: i64,
    pub dex: i64,
    pub crit_pct: i64,
    pub crit_damage_pct: i64,
    pub control_resist_pct: i64,
    pub life_steal_pct: i64,
    pub mana_steal_pct: i64,
    pub damage_reduce_pct: i64,
    pub atk_pct: i64,
    pub def_pct: i64,
    pub hp_pct: i64,
    pub mp_pct: i64,
}

#[derive(Debug, Clone, FromRow)]
struct AdventureBuffRow {
    stat: String,
    pct: i32,
}

impl SystemsRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn ensure_starter(&self, character_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into pets (character_id, template_id, name, level, fighting, skills)
            select $1, pt.id, pt.name, 1, true, pt.skills
            from pet_templates pt
            where pt.id = 'pet_white_tiger'
              and not exists (select 1 from pets where character_id = $1)
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            insert into treasures (character_id, template_id, level, stage, equipped)
            select $1, tt.id, 1, 0, true
            from treasure_templates tt
            where tt.id = 'treasure_dragon_seal'
              and not exists (select 1 from treasures where character_id = $1)
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        self.ensure_cultivation(character_id).await?;
        self.ensure_wanxiang(character_id).await?;
        Ok(())
    }

    pub async fn upgrade_pet(
        &self,
        character_id: i64,
        target_id: Option<i64>,
    ) -> Result<SystemActionResult, SystemsActionError> {
        self.ensure_starter(character_id).await?;
        if !self.system_unlocked(character_id, "pet").await? {
            return Err(SystemsActionError::Locked("宠物需要击杀尸傀监工后开启。".into()));
        }
        let mut tx = self.pool.begin().await?;
        let character = lock_character(&mut tx, character_id).await?;
        let pet = sqlx::query_as::<_, PetUpgradeRow>(
            r#"
            select id, level
            from pets
            where character_id = $1 and ($2::bigint is null or id = $2)
            order by fighting desc, level desc, id asc
            limit 1
            for update
            "#,
        )
        .bind(character_id)
        .bind(target_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(SystemsActionError::NotFound)?;
        let max_level = character.level.clamp(1, PET_MAX_LEVEL);
        if pet.level >= max_level {
            return Err(SystemsActionError::MaxLevel);
        }
        let required_food = growth_material_cost(pet.level, PET_MAX_LEVEL);
        let gold_cost = growth_gold_cost(pet.level, 15_000, 100);
        debit_gold(&mut tx, character_id, gold_cost).await?;
        consume_stack(&mut tx, character_id, "pet_food", required_food).await?;
        sqlx::query("update pets set level = level + 1, exp = exp + $2 where id = $1")
            .bind(pet.id)
            .bind(required_food * 100)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        let character = refreshed_character(&self.pool, character_id).await?;
        Ok(SystemActionResult {
            systems: self.overview(character_id).await?,
            character: character_view(character),
            message: format!("宠物升级成功，消耗灵兽粮 {} 个、金币 {}。", required_food, gold_cost),
        })
    }

    pub async fn upgrade_treasure(
        &self,
        character_id: i64,
        target_id: Option<i64>,
    ) -> Result<SystemActionResult, SystemsActionError> {
        self.ensure_starter(character_id).await?;
        if !self.system_unlocked(character_id, "treasure").await? {
            return Err(SystemsActionError::Locked("法宝需要击杀狂暴猪王后开启。".into()));
        }
        let mut tx = self.pool.begin().await?;
        lock_character(&mut tx, character_id).await?;
        let treasure = sqlx::query_as::<_, TreasureUpgradeRow>(
            r#"
            select id, level, stage
            from treasures
            where character_id = $1 and ($2::bigint is null or id = $2)
            order by equipped desc, stage desc, level desc, id asc
            limit 1
            for update
            "#,
        )
        .bind(character_id)
        .bind(target_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(SystemsActionError::NotFound)?;
        if treasure.level >= TREASURE_MAX_LEVEL {
            return Err(SystemsActionError::MaxLevel);
        }
        let required_shards = growth_material_cost(treasure.level, TREASURE_MAX_LEVEL);
        let gold_cost = growth_gold_cost(treasure.level, 10_000, 90);
        let next_level = treasure.level + 1;
        let next_stage = treasure.stage.max((next_level + 9) / 10).clamp(1, TREASURE_MAX_STAGE);
        debit_gold(&mut tx, character_id, gold_cost).await?;
        consume_stack(&mut tx, character_id, "treasure_shard", required_shards).await?;
        sqlx::query("update treasures set level = $2, stage = $3 where id = $1")
            .bind(treasure.id)
            .bind(next_level)
            .bind(next_stage)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        let character = refreshed_character(&self.pool, character_id).await?;
        Ok(SystemActionResult {
            systems: self.overview(character_id).await?,
            character: character_view(character),
            message: format!("法宝升级成功，消耗法宝碎片 {} 个、金币 {}。", required_shards, gold_cost),
        })
    }

    pub async fn cultivate(&self, character_id: i64) -> Result<SystemActionResult, SystemsActionError> {
        self.ensure_starter(character_id).await?;
        if !self.system_unlocked(character_id, "cultivation").await? {
            return Err(SystemsActionError::Locked("境界需要击杀镇界石魔后开启。".into()));
        }
        let mut tx = self.pool.begin().await?;
        let character = lock_character(&mut tx, character_id).await?;
        let state = sqlx::query_as::<_, CultivationStateRow>(
            "select layer, progress from cultivation_states where character_id = $1 for update",
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;
        let max_layer = character.level.clamp(1, CULTIVATION_MAX_LAYER);
        if state.layer >= max_layer {
            return Err(SystemsActionError::MaxLevel);
        }
        let required_pills = growth_material_cost(state.layer, CULTIVATION_MAX_LAYER);
        let gold_cost = growth_gold_cost(state.layer, 100_000, 12_000);
        debit_gold(&mut tx, character_id, gold_cost).await?;
        consume_stack(&mut tx, character_id, "cultivation_pill", required_pills).await?;
        sqlx::query("update cultivation_states set layer = layer + 1, progress = progress + $2 where character_id = $1")
            .bind(character_id)
            .bind(required_pills * 100)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        let character = refreshed_character(&self.pool, character_id).await?;
        Ok(SystemActionResult {
            systems: self.overview(character_id).await?,
            character: character_view(character),
            message: format!("修炼突破成功，消耗修炼丹 {} 个、金币 {}。", required_pills, gold_cost),
        })
    }

    pub async fn upgrade_wanxiang(&self, character_id: i64) -> Result<SystemActionResult, SystemsActionError> {
        self.ensure_starter(character_id).await?;
        self.unlock_for_current_position(character_id).await?;
        if !self.system_unlocked(character_id, "wanxiang").await? {
            return Err(SystemsActionError::Locked("万象铸体需要首次抵达星际观测台后开启。".into()));
        }
        let mut tx = self.pool.begin().await?;
        lock_character(&mut tx, character_id).await?;
        let body = sqlx::query_as::<_, WanxiangBodyRow>(
            r#"
            select level, essence
            from character_wanxiang_bodies
            where character_id = $1
            for update
            "#,
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;
        if body.level >= WANXIANG_MAX_LEVEL {
            return Err(SystemsActionError::MaxLevel);
        }
        let cost = wanxiang_upgrade_cost(body.level);
        if body.essence < cost {
            return Err(SystemsActionError::NotEnoughMaterial);
        }
        let fail_pct = wanxiang_fail_pct(body.level);
        let failed = fail_pct > 0 && thread_rng().gen_range(0..100) < fail_pct;
        let next_level = if failed { body.level } else { body.level + 1 };
        sqlx::query(
            r#"
            update character_wanxiang_bodies
            set level = $2, essence = essence - $3, updated_at = now()
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(next_level)
        .bind(cost)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        let character = refreshed_character(&self.pool, character_id).await?;
        let message = if failed {
            format!("万象铸体失败，消耗灵韵 {} 点。当前失败率 {}%。", cost, fail_pct)
        } else {
            format!("万象铸体成功，提升至 Lv.{}，消耗灵韵 {} 点。", next_level, cost)
        };
        Ok(SystemActionResult {
            systems: self.overview(character_id).await?,
            character: character_view(character),
            message,
        })
    }

    pub async fn extract_wanxiang_essence(
        &self,
        character_id: i64,
        max_tier: i32,
    ) -> Result<SystemActionResult, SystemsActionError> {
        self.ensure_starter(character_id).await?;
        self.unlock_for_current_position(character_id).await?;
        if !self.system_unlocked(character_id, "wanxiang").await? {
            return Err(SystemsActionError::Locked("万象铸体需要首次抵达星际观测台后开启。".into()));
        }
        let max_tier = max_tier.clamp(1, 17);
        let mut tx = self.pool.begin().await?;
        lock_character(&mut tx, character_id).await?;
        let (item_count, essence_total): (i64, i64) = sqlx::query_as(
            r#"
            select
              count(ii.id)::bigint as item_count,
              coalesce(
                sum(
                  ii.quantity * case
                    when coalesce(it.stats->>'set', '') = 'dominator' or it.id like 'dominator_%' then 100
                    else coalesce(nullif(it.stats->>'tier', '')::bigint, 0)
                  end
                ),
                0
              )::bigint as essence_total
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location = 'bag'
              and it.slot is not null
              and coalesce(nullif(it.stats->>'tier', '')::int, 0) between 1 and $2
            "#,
        )
        .bind(character_id)
        .bind(max_tier)
        .fetch_one(&mut *tx)
        .await?;
        if essence_total > 0 {
            sqlx::query(
                r#"
                delete from inventory_items ii
                using item_templates it
                where ii.template_id = it.id
                  and ii.character_id = $1
                  and ii.location = 'bag'
                  and it.slot is not null
                  and coalesce(nullif(it.stats->>'tier', '')::int, 0) between 1 and $2
                "#,
            )
            .bind(character_id)
            .bind(max_tier)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                r#"
                insert into character_wanxiang_bodies (character_id, level, essence)
                values ($1, 1, $2)
                on conflict (character_id) do update
                set essence = character_wanxiang_bodies.essence + excluded.essence,
                    updated_at = now()
                "#,
            )
            .bind(character_id)
            .bind(essence_total)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        let character = refreshed_character(&self.pool, character_id).await?;
        let message = if essence_total > 0 {
            format!("灵韵提取完成，熔炼 {} 件装备，获得 {} 点灵韵。", item_count, essence_total)
        } else {
            format!("背包中没有 1-{} 阶可提取装备。", max_tier)
        };
        Ok(SystemActionResult {
            systems: self.overview(character_id).await?,
            character: character_view(character),
            message,
        })
    }

    pub async fn combat_bonus(&self, character_id: i64) -> Result<SystemBonus, sqlx::Error> {
        self.ensure_starter(character_id).await?;
        let mut bonus = SystemBonus::default();
        let unlocks = self.unlocks(character_id).await?;

        if unlocks.pet.unlocked {
            if let Some(pet) = sqlx::query_as::<_, PetCombatRow>(
                r#"
                select p.level
                from pets p
                where p.character_id = $1 and p.fighting = true
                order by p.level desc, p.id asc
                limit 1
                "#,
            )
            .bind(character_id)
            .fetch_optional(&self.pool)
            .await?
            {
                let level = i64::from(pet.level.clamp(1, PET_MAX_LEVEL));
                bonus.atk += scaled_bonus(level, 2_600, i64::from(PET_MAX_LEVEL));
                bonus.mag += scaled_bonus(level, 2_300, i64::from(PET_MAX_LEVEL));
                bonus.hp += scaled_bonus(level, 16_000, i64::from(PET_MAX_LEVEL));
                bonus.def += scaled_bonus(level, 600, i64::from(PET_MAX_LEVEL));
                bonus.mdef += scaled_bonus(level, 600, i64::from(PET_MAX_LEVEL));
            }
        }

        if unlocks.treasure.unlocked {
            let treasures = sqlx::query_as::<_, TreasureCombatRow>(
                r#"
                select t.level, t.stage
                from treasures t
                where t.character_id = $1 and t.equipped = true
                "#,
            )
            .bind(character_id)
            .fetch_all(&self.pool)
            .await?;
            for treasure in treasures {
                let level = i64::from(treasure.level.clamp(1, TREASURE_MAX_LEVEL));
                let stage = i64::from(treasure.stage.clamp(1, TREASURE_MAX_STAGE));
                bonus.atk += level * 10 + stage * 40;
                bonus.mag += level * 10 + stage * 40;
                bonus.hp += level * 60 + stage * 200;
                bonus.crit_pct += level / 20 + stage * 2;
            }
        }

        if unlocks.cultivation.unlocked {
            if let Some(cultivation) = sqlx::query_as::<_, CultivationStateRow>(
                "select layer, progress from cultivation_states where character_id = $1",
            )
            .bind(character_id)
            .fetch_optional(&self.pool)
            .await?
            {
                let layer = i64::from(cultivation.layer.clamp(1, CULTIVATION_MAX_LAYER));
                let max_layer = i64::from(CULTIVATION_MAX_LAYER);
                bonus.atk += scaled_bonus(layer, 3_200, max_layer);
                bonus.mag += scaled_bonus(layer, 3_200, max_layer);
                bonus.hp += scaled_bonus(layer, 32_000, max_layer);
                bonus.mp += scaled_bonus(layer, 32_000, max_layer);
                bonus.def += scaled_bonus(layer, 2_000, max_layer);
                bonus.mdef += scaled_bonus(layer, 2_000, max_layer);
            }
        }

        if unlocks.wanxiang.unlocked {
            if let Some(body) = self.wanxiang_row(character_id).await? {
                let stats = wanxiang_stats(body.level);
                bonus.atk += stats.atk;
                bonus.mag += stats.mag;
                bonus.hp += stats.hp;
                bonus.mp += stats.mp;
                bonus.def += stats.def;
                bonus.mdef += stats.mdef;
                bonus.life_steal_pct += stats.life_steal_pct;
                bonus.mana_steal_pct += stats.mana_steal_pct;
                bonus.damage_reduce_pct += stats.damage_reduce_pct;
            }
        }

        let totems = sqlx::query_as::<_, GuildTotemCombatRow>(
            r#"
            select gt.totem, gt.level
            from guild_totems gt
            join guild_members gm on gm.guild_id = gt.guild_id and gm.character_id = gt.character_id
            where gt.character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        for totem in totems {
            let level = i64::from(totem.level.clamp(0, 100));
            match totem.totem.as_str() {
                "qiongqi" => {
                    bonus.atk += level * 100;
                    bonus.mag += level * 100;
                }
                "bifang" => {
                    bonus.crit_pct += level / 5;
                    bonus.crit_damage_pct += level;
                }
                "chenghuang" => {
                    bonus.hp += level * 500;
                    bonus.mp += level * 500;
                }
                "xuangui" => {
                    bonus.def += level * 50;
                    bonus.mdef += level * 50;
                }
                _ => {}
            }
        }

        let adventure_buffs = sqlx::query_as::<_, AdventureBuffRow>(
            r#"
            select stat, pct
            from character_adventure_buffs
            where character_id = $1 and expires_at > now()
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        for buff in adventure_buffs {
            let pct = i64::from(buff.pct);
            match buff.stat.as_str() {
                "atk" => bonus.atk_pct += pct,
                "hp" => {
                    bonus.hp_pct += pct;
                    bonus.mp_pct += pct;
                }
                "def" => bonus.def_pct += pct,
                _ => {}
            }
        }

        Ok(bonus)
    }

    pub async fn overview(&self, character_id: i64) -> Result<PlayerSystemsView, sqlx::Error> {
        self.ensure_starter(character_id).await?;
        self.unlock_for_current_position(character_id).await?;
        let pets = self.pets(character_id).await?;
        let treasures = self.treasures(character_id).await?;
        let vip = self.vip(character_id).await?;
        let vip_settings = self.vip_potion_settings(character_id).await?;
        let unlocks = self.unlocks(character_id).await?;
        let cultivation = self.cultivation(character_id).await?;
        let wanxiang = if unlocks.wanxiang.unlocked {
            self.wanxiang(character_id).await?
        } else {
            None
        };
        let sabak = self.sabak().await?;
        Ok(PlayerSystemsView {
            pets,
            treasures,
            vip,
            vip_settings,
            cultivation,
            wanxiang,
            unlocks,
            sabak,
        })
    }

    pub async fn unlock_for_mob(
        &self,
        character_id: i64,
        mob_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let rules = unlock_rules_for_mob(mob_id);
        if rules.is_empty() {
            return Ok(None);
        }
        let mut messages = Vec::new();
        for rule in rules {
            let inserted = sqlx::query(
                r#"
                insert into character_system_unlocks (character_id, system_id, source_mob_id)
                values ($1, $2, $3)
                on conflict (character_id, system_id) do nothing
                "#,
            )
            .bind(character_id)
            .bind(rule.system_id)
            .bind(mob_id)
            .execute(&self.pool)
            .await?;
            if inserted.rows_affected() > 0 {
                messages.push(format!("{}已开启：{}。", rule.name, rule.message));
            }
        }
        Ok((!messages.is_empty()).then(|| messages.join(" ")))
    }

    pub async fn unlock_for_position(
        &self,
        character_id: i64,
        zone: &str,
        room: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        if zone != "ancient_secret" || room != "stargazer_observatory" {
            return Ok(None);
        }
        let rule = unlock_rule("wanxiang");
        let inserted = sqlx::query(
            r#"
            insert into character_system_unlocks (character_id, system_id, source_mob_id)
            values ($1, $2, $3)
            on conflict (character_id, system_id) do nothing
            "#,
        )
        .bind(character_id)
        .bind(rule.system_id)
        .bind("stargazer_observatory")
        .execute(&self.pool)
        .await?;
        Ok((inserted.rows_affected() > 0).then(|| format!("{}已开启：{}。", rule.name, rule.message)))
    }

    pub async fn record_stargazer_entry(&self, character_id: i64) -> Result<Option<String>, sqlx::Error> {
        let (visits, awarded): (i64, bool) = sqlx::query_as(
            r#"
            insert into character_stargazer_visits (character_id, visits)
            values ($1, 1)
            on conflict (character_id) do update
            set visits = character_stargazer_visits.visits + 1,
                updated_at = now()
            returning visits, awarded
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        if visits < 1000 || awarded {
            return Ok(None);
        }
        let (already_has,): (bool,) = sqlx::query_as(
            r#"
            select exists(
              select 1
              from inventory_items
              where character_id = $1
                and template_id = 'bracelet_star_devourer'
                and location in ('bag', 'warehouse', 'equipped')
            )
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        if !already_has {
            sqlx::query(
                r#"
                insert into inventory_items (character_id, template_id, quantity, location, bind)
                values ($1, 'bracelet_star_devourer', 1, 'bag', true)
                "#,
            )
            .bind(character_id)
            .execute(&self.pool)
            .await?;
        }
        sqlx::query(
            r#"
            update character_stargazer_visits
            set awarded = true,
                updated_at = now()
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        Ok(Some(format!(
            "星际观测台共鸣达成：累计进入 {} 次，获得噬星镯。",
            visits
        )))
    }

    pub async fn unlock_for_current_position(&self, character_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query_as::<_, (String, String)>(
            r#"
            select zone, room
            from character_state
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        if let Some((zone, room)) = row {
            self.unlock_for_position(character_id, &zone, &room).await
        } else {
            Ok(None)
        }
    }

    pub async fn has_active_vip(&self, character_id: i64) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            select count(*)::bigint
            from vip_records
            where character_id = $1
              and (ends_at is null or ends_at > now())
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn vip_potion_settings(&self, character_id: i64) -> Result<PlayerVipSettingsView, sqlx::Error> {
        sqlx::query(
            r#"
            insert into vip_potion_settings (character_id)
            values ($1)
            on conflict (character_id) do nothing
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, VipSettingsRow>(
            r#"
            select hp_enabled, hp_threshold_pct, hp_template_id,
                   mp_enabled, mp_threshold_pct, mp_template_id,
                   auto_decompose_enabled, auto_decompose_max_tier,
                   auto_extract_essence_enabled, auto_extract_essence_max_tier
            from vip_potion_settings
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(vip_settings_view(row))
    }

    pub async fn update_vip_potion_settings(
        &self,
        character_id: i64,
        input: VipPotionSettingsRequest,
    ) -> Result<PlayerVipSettingsView, SystemsActionError> {
        if input.auto_decompose_enabled && input.auto_extract_essence_enabled {
            return Err(SystemsActionError::Locked("自动拆解低阶装备和自动提取灵韵只能开启一个。".into()));
        }
        ensure_consumable(&self.pool, input.hp_template_id.trim()).await?;
        ensure_consumable(&self.pool, input.mp_template_id.trim()).await?;
        let row = sqlx::query_as::<_, VipSettingsRow>(
            r#"
            insert into vip_potion_settings (
              character_id,
              hp_enabled,
              hp_threshold_pct,
              hp_template_id,
              mp_enabled,
              mp_threshold_pct,
              mp_template_id,
              auto_decompose_enabled,
              auto_decompose_max_tier,
              auto_extract_essence_enabled,
              auto_extract_essence_max_tier
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            on conflict (character_id) do update set
              hp_enabled = excluded.hp_enabled,
              hp_threshold_pct = excluded.hp_threshold_pct,
              hp_template_id = excluded.hp_template_id,
              mp_enabled = excluded.mp_enabled,
              mp_threshold_pct = excluded.mp_threshold_pct,
              mp_template_id = excluded.mp_template_id,
              auto_decompose_enabled = excluded.auto_decompose_enabled,
              auto_decompose_max_tier = excluded.auto_decompose_max_tier,
              auto_extract_essence_enabled = excluded.auto_extract_essence_enabled,
              auto_extract_essence_max_tier = excluded.auto_extract_essence_max_tier
            returning hp_enabled, hp_threshold_pct, hp_template_id,
                      mp_enabled, mp_threshold_pct, mp_template_id,
                      auto_decompose_enabled, auto_decompose_max_tier,
                      auto_extract_essence_enabled, auto_extract_essence_max_tier
            "#,
        )
        .bind(character_id)
        .bind(input.hp_enabled)
        .bind(input.hp_threshold_pct.clamp(1, 99))
        .bind(input.hp_template_id.trim())
        .bind(input.mp_enabled)
        .bind(input.mp_threshold_pct.clamp(1, 99))
        .bind(input.mp_template_id.trim())
        .bind(input.auto_decompose_enabled)
        .bind(input.auto_decompose_max_tier.clamp(0, 17))
        .bind(input.auto_extract_essence_enabled)
        .bind(input.auto_extract_essence_max_tier.clamp(0, 17))
        .fetch_one(&self.pool)
        .await?;
        Ok(vip_settings_view(row))
    }

    pub async fn redeem_card(
        &self,
        character_id: i64,
        code: &str,
    ) -> Result<RechargeCardResult, SystemsActionError> {
        let code = code.trim();
        if code.is_empty() {
            return Err(SystemsActionError::NotFound);
        }
        let mut tx = self.pool.begin().await?;
        let card = sqlx::query_as::<_, RechargeCardRow>(
            r#"
            select id, yuanbao
            from recharge_cards
            where code = $1 and used_by is null and used_at is null
            for update
            "#,
        )
        .bind(code)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(SystemsActionError::NotFound)?;
        let character = sqlx::query_as::<_, CharacterRecord>(
            r#"
            update characters
            set yuanbao = yuanbao + $2
            where id = $1 and deleted_at is null
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(character_id)
        .bind(card.yuanbao.max(0))
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(SystemsActionError::NotFound)?;
        sqlx::query("update recharge_cards set used_by = $2, used_at = now() where id = $1")
            .bind(card.id)
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
        let tier = if card.yuanbao >= 500 { "svip" } else { "vip" };
        if card.yuanbao >= 100 {
            sqlx::query(
                r#"
                insert into vip_records (character_id, tier, starts_at, ends_at)
                values ($1, $2, now(), now() + interval '7 days')
                "#,
            )
            .bind(character_id)
            .bind(tier)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(RechargeCardResult {
            character: character_view(character),
            vip: self.vip(character_id).await?,
            message: format!("兑换成功，获得 {} 元宝{}。", card.yuanbao, if card.yuanbao >= 100 { "并激活会员" } else { "" }),
        })
    }

    async fn pets(&self, character_id: i64) -> Result<Vec<PlayerPetView>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PetRow>(
            r#"
            select
              p.id,
              p.template_id,
              p.name,
              pt.rarity,
              p.level,
              p.exp,
              p.fighting,
              pt.base_hp,
              pt.base_atk,
              p.skills
            from pets p
            join pet_templates pt on pt.id = p.template_id
            where p.character_id = $1
            order by p.fighting desc, p.level desc, p.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(pet_view).collect())
    }

    async fn treasures(&self, character_id: i64) -> Result<Vec<PlayerTreasureView>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TreasureRow>(
            r#"
            select
              t.id,
              t.template_id,
              tt.name,
              tt.family,
              tt.passive,
              t.level,
              t.stage,
              t.equipped,
              tt.config
            from treasures t
            join treasure_templates tt on tt.id = t.template_id
            where t.character_id = $1
            order by t.equipped desc, t.stage desc, t.level desc, t.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(treasure_view).collect())
    }

    async fn vip(&self, character_id: i64) -> Result<Option<PlayerVipView>, sqlx::Error> {
        let row = sqlx::query_as::<_, VipRow>(
            r#"
            select tier, starts_at, ends_at
            from vip_records
            where character_id = $1
              and (ends_at is null or ends_at > now())
            order by
              case tier when 'permanent_svip' then 3 when 'svip' then 2 else 1 end desc,
              created_at desc
            limit 1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| PlayerVipView {
            tier: row.tier,
            starts_at: fmt_time(row.starts_at),
            ends_at: row.ends_at.map(fmt_time),
        }))
    }

    async fn cultivation(&self, character_id: i64) -> Result<Option<PlayerCultivationView>, sqlx::Error> {
        self.ensure_cultivation(character_id).await?;
        let row = sqlx::query_as::<_, CultivationRow>(
            r#"
            select c.level, c.exp, cs.layer, cs.progress
            from characters c
            join cultivation_states cs on cs.character_id = c.id
            where c.id = $1 and c.deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(cultivation_view))
    }

    async fn wanxiang(&self, character_id: i64) -> Result<Option<PlayerWanxiangBodyView>, sqlx::Error> {
        self.ensure_wanxiang(character_id).await?;
        Ok(self.wanxiang_row(character_id).await?.map(wanxiang_view))
    }

    async fn wanxiang_row(&self, character_id: i64) -> Result<Option<WanxiangBodyRow>, sqlx::Error> {
        sqlx::query_as::<_, WanxiangBodyRow>(
            r#"
            select level, essence
            from character_wanxiang_bodies
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn unlocks(&self, character_id: i64) -> Result<PlayerSystemUnlocksView, sqlx::Error> {
        let rows = sqlx::query_as::<_, SystemUnlockRow>(
            r#"
            select system_id, unlocked_at::text as unlocked_at
            from character_system_unlocks
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(PlayerSystemUnlocksView {
            pet: system_unlock_view("pet", &rows),
            treasure: system_unlock_view("treasure", &rows),
            cultivation: system_unlock_view("cultivation", &rows),
            wanxiang: system_unlock_view("wanxiang", &rows),
        })
    }

    async fn system_unlocked(&self, character_id: i64, system_id: &str) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            select count(*)::bigint
            from character_system_unlocks
            where character_id = $1 and system_id = $2
            "#,
        )
        .bind(character_id)
        .bind(system_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    async fn ensure_cultivation(&self, character_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into cultivation_states (character_id, layer, progress)
            values ($1, 1, 0)
            on conflict (character_id) do nothing
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_wanxiang(&self, character_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into character_wanxiang_bodies (character_id, level, essence)
            values ($1, 1, 0)
            on conflict (character_id) do nothing
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn sabak(&self) -> Result<Option<PlayerSabakView>, sqlx::Error> {
        let row = sqlx::query_as::<_, SabakRow>(
            r#"
            with schedule as (
              select
                now() as current_at,
                (date_trunc('week', now() at time zone 'Asia/Shanghai') + interval '6 days 20 hours') at time zone 'Asia/Shanghai' as this_week_battle
            ),
            next_battle as (
              select
                case
                  when current_at <= this_week_battle then this_week_battle
                  else this_week_battle + interval '7 days'
                end as battle_at
              from schedule
            ),
            owner as (
              select coalesce(
                (select name from guilds where sabak_owner = true order by id asc limit 1),
                (select winner_name from guild_sabak_state where id = 1),
                '比奇远征队'
              ) as name
            )
            select
              1::bigint as campaign_id,
              '自动报名'::text as status,
              now() as signup_starts_at,
              nb.battle_at as battle_starts_at,
              nb.battle_at + interval '5 minutes' as battle_ends_at,
              owner.name as defending_guild,
              owner.name as winner_guild,
              (select count(*)::bigint from guilds where level >= 20) as signup_count,
              0::int as tax_rate_pct
            from next_battle nb
            cross join owner
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| PlayerSabakView {
            campaign_id: row.campaign_id,
            status: row.status,
            signup_starts_at: fmt_time(row.signup_starts_at),
            battle_starts_at: fmt_time(row.battle_starts_at),
            battle_ends_at: fmt_time(row.battle_ends_at),
            defending_guild: row.defending_guild,
            winner_guild: row.winner_guild,
            signup_count: row.signup_count,
            tax_rate_pct: row.tax_rate_pct,
        }))
    }
}

#[derive(Debug, FromRow)]
struct RechargeCardRow {
    id: i64,
    yuanbao: i64,
}

#[derive(Debug, FromRow)]
struct PetRow {
    id: i64,
    template_id: String,
    name: String,
    rarity: String,
    level: i32,
    exp: i64,
    fighting: bool,
    base_hp: i64,
    base_atk: i64,
    skills: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct PetUpgradeRow {
    id: i64,
    level: i32,
}

#[derive(Debug, FromRow)]
struct PetCombatRow {
    level: i32,
}

#[derive(Debug, FromRow)]
struct TreasureRow {
    id: i64,
    template_id: String,
    name: String,
    family: String,
    passive: String,
    level: i32,
    stage: i32,
    equipped: bool,
    config: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct TreasureUpgradeRow {
    id: i64,
    level: i32,
    stage: i32,
}

#[derive(Debug, FromRow)]
struct TreasureCombatRow {
    level: i32,
    stage: i32,
}

#[derive(Debug, FromRow)]
struct GuildTotemCombatRow {
    totem: String,
    level: i32,
}

#[derive(Debug, FromRow)]
struct VipRow {
    tier: String,
    starts_at: OffsetDateTime,
    ends_at: Option<OffsetDateTime>,
}

#[derive(Debug, FromRow)]
struct VipSettingsRow {
    hp_enabled: bool,
    hp_threshold_pct: i32,
    hp_template_id: String,
    mp_enabled: bool,
    mp_threshold_pct: i32,
    mp_template_id: String,
    auto_decompose_enabled: bool,
    auto_decompose_max_tier: i32,
    auto_extract_essence_enabled: bool,
    auto_extract_essence_max_tier: i32,
}

#[derive(Debug, FromRow)]
struct CultivationRow {
    level: i32,
    exp: i64,
    layer: i32,
    progress: i64,
}

#[derive(Debug, FromRow)]
struct CultivationStateRow {
    layer: i32,
    progress: i64,
}

#[derive(Debug, FromRow)]
struct WanxiangBodyRow {
    level: i32,
    essence: i64,
}

#[derive(Debug, Clone, Copy, Default)]
struct WanxiangStats {
    atk: i64,
    mag: i64,
    hp: i64,
    mp: i64,
    def: i64,
    mdef: i64,
    life_steal_pct: i64,
    mana_steal_pct: i64,
    damage_reduce_pct: i64,
}

#[derive(Debug, FromRow)]
struct SabakRow {
    campaign_id: i64,
    status: String,
    signup_starts_at: OffsetDateTime,
    battle_starts_at: OffsetDateTime,
    battle_ends_at: OffsetDateTime,
    defending_guild: Option<String>,
    winner_guild: Option<String>,
    signup_count: i64,
    tax_rate_pct: i32,
}

#[derive(Debug, FromRow)]
struct SystemUnlockRow {
    system_id: String,
    unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct SystemUnlockRule {
    system_id: &'static str,
    name: &'static str,
    requirement: &'static str,
    source: &'static str,
    message: &'static str,
}

fn fmt_time(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).unwrap_or_else(|_| value.to_string())
}

fn system_unlock_view(system_id: &str, rows: &[SystemUnlockRow]) -> PlayerSystemUnlockView {
    let rule = unlock_rule(system_id);
    let row = rows.iter().find(|row| row.system_id == system_id);
    PlayerSystemUnlockView {
        unlocked: row.is_some(),
        requirement: rule.requirement.into(),
        source: rule.source.into(),
        unlocked_at: row.and_then(|row| row.unlocked_at.clone()),
    }
}

fn unlock_rule(system_id: &str) -> SystemUnlockRule {
    match system_id {
        "treasure" => SystemUnlockRule {
            system_id: "treasure",
            name: "法宝",
            requirement: "击杀狂暴猪王",
            source: "狂暴猪王",
            message: "击杀狂暴猪王后，法宝系统开放",
        },
        "cultivation" => SystemUnlockRule {
            system_id: "cultivation",
            name: "境界",
            requirement: "击杀镇界石魔",
            source: "镇界石魔",
            message: "击杀镇界石魔后，境界修炼开放",
        },
        "wanxiang" => SystemUnlockRule {
            system_id: "wanxiang",
            name: "万象铸体",
            requirement: "首次进入星际观测台",
            source: "星际观测台",
            message: "抵达星际观测台后，万化神炉与万象铸体开放",
        },
        _ => SystemUnlockRule {
            system_id: "pet",
            name: "宠物",
            requirement: "击杀尸傀监工",
            source: "尸傀监工",
            message: "击杀尸傀监工后，宠物系统开放",
        },
    }
}

fn unlock_rules_for_mob(mob_id: &str) -> Vec<SystemUnlockRule> {
    match mob_id {
        "boss_raging_boar_king" => vec![unlock_rule("treasure")],
        "boss_corpse_foreman" => vec![unlock_rule("pet")],
        "boss_boundary_stonemaw" => vec![unlock_rule("cultivation")],
        _ => Vec::new(),
    }
}

fn pet_view(row: PetRow) -> PlayerPetView {
    PlayerPetView {
        id: row.id,
        template_id: row.template_id,
        name: row.name,
        rarity: row.rarity,
        level: row.level,
        exp: row.exp,
        fighting: row.fighting,
        base_hp: row.base_hp,
        base_atk: row.base_atk,
        skills: row.skills,
    }
}

fn treasure_view(row: TreasureRow) -> PlayerTreasureView {
    PlayerTreasureView {
        id: row.id,
        template_id: row.template_id,
        name: row.name,
        family: row.family,
        passive: row.passive,
        level: row.level,
        stage: row.stage,
        equipped: row.equipped,
        config: row.config,
    }
}

fn vip_settings_view(row: VipSettingsRow) -> PlayerVipSettingsView {
    PlayerVipSettingsView {
        hp_enabled: row.hp_enabled,
        hp_threshold_pct: row.hp_threshold_pct,
        hp_template_id: row.hp_template_id,
        mp_enabled: row.mp_enabled,
        mp_threshold_pct: row.mp_threshold_pct,
        mp_template_id: row.mp_template_id,
        auto_decompose_enabled: row.auto_decompose_enabled,
        auto_decompose_max_tier: row.auto_decompose_max_tier,
        auto_extract_essence_enabled: row.auto_extract_essence_enabled,
        auto_extract_essence_max_tier: row.auto_extract_essence_max_tier,
    }
}

fn cultivation_view(row: CultivationRow) -> PlayerCultivationView {
    let layer_total = row.layer.clamp(1, CULTIVATION_MAX_LAYER);
    let realm_index = (layer_total - 1) / 9;
    let layer = (layer_total - 1) % 9 + 1;
    let realm = match realm_index {
        0 => "淬体",
        1 => "凝神",
        2 => "练气",
        3 => "元婴",
        4 => "登仙",
        5 => "化神",
        6 => "太初",
        7 => "创世",
        _ => "主宰",
    };
    let next_level_exp = if layer_total >= CULTIVATION_MAX_LAYER {
        0
    } else {
        growth_material_cost(layer_total, CULTIVATION_MAX_LAYER)
    };
    let progress_pct = (i64::from(layer_total) * 100 / i64::from(CULTIVATION_MAX_LAYER)).clamp(0, 100) as i32;
    PlayerCultivationView {
        realm: realm.into(),
        layer,
        next_level_exp,
        progress_pct,
    }
}

fn wanxiang_view(row: WanxiangBodyRow) -> PlayerWanxiangBodyView {
    let level = row.level.clamp(1, WANXIANG_MAX_LEVEL);
    let stats = wanxiang_stats(level);
    PlayerWanxiangBodyView {
        level,
        essence: row.essence,
        next_cost: if level >= WANXIANG_MAX_LEVEL {
            0
        } else {
            wanxiang_upgrade_cost(level)
        },
        fail_pct: wanxiang_fail_pct(level),
        progress_pct: (i64::from(level) * 100 / i64::from(WANXIANG_MAX_LEVEL)).clamp(0, 100) as i32,
        atk: stats.atk,
        mag: stats.mag,
        hp: stats.hp,
        mp: stats.mp,
        def: stats.def,
        mdef: stats.mdef,
        life_steal_pct: stats.life_steal_pct,
        mana_steal_pct: stats.mana_steal_pct,
        damage_reduce_pct: stats.damage_reduce_pct,
    }
}

fn wanxiang_stats(level: i32) -> WanxiangStats {
    let level = level.clamp(1, WANXIANG_MAX_LEVEL);
    if level >= WANXIANG_MAX_LEVEL {
        return WanxiangStats {
            atk: WANXIANG_FULL_ATK,
            mag: WANXIANG_FULL_MAG,
            hp: WANXIANG_FULL_HP,
            mp: WANXIANG_FULL_MP,
            def: WANXIANG_FULL_DEF,
            mdef: WANXIANG_FULL_MDEF,
            life_steal_pct: WANXIANG_FULL_LIFE_STEAL_PCT,
            mana_steal_pct: WANXIANG_FULL_MANA_STEAL_PCT,
            damage_reduce_pct: WANXIANG_FULL_DAMAGE_REDUCE_PCT,
        };
    }
    let scale = i64::from(level);
    let denominator = 1_998_i64;
    WanxiangStats {
        atk: WANXIANG_FULL_ATK * scale / denominator,
        mag: WANXIANG_FULL_MAG * scale / denominator,
        hp: WANXIANG_FULL_HP * scale / denominator,
        mp: WANXIANG_FULL_MP * scale / denominator,
        def: WANXIANG_FULL_DEF * scale / denominator,
        mdef: WANXIANG_FULL_MDEF * scale / denominator,
        life_steal_pct: WANXIANG_FULL_LIFE_STEAL_PCT * scale / denominator,
        mana_steal_pct: WANXIANG_FULL_MANA_STEAL_PCT * scale / denominator,
        damage_reduce_pct: WANXIANG_FULL_DAMAGE_REDUCE_PCT * scale / denominator,
    }
}

fn wanxiang_upgrade_cost(level: i32) -> i64 {
    let level = i64::from(level.clamp(1, WANXIANG_MAX_LEVEL));
    level.saturating_mul(level).saturating_mul(2) / 100 + level.saturating_mul(5) + 20
}

fn wanxiang_fail_pct(level: i32) -> i32 {
    match level {
        990..=999 => 99,
        900..=989 => 90,
        500..=899 => 20 + ((level - 500) / 100) * 10,
        _ => 0,
    }
}

fn growth_material_cost(current_rank: i32, max_rank: i32) -> i64 {
    let max_rank = max_rank.max(2);
    let steps = i64::from(max_rank - 1);
    let current = i64::from(current_rank.clamp(1, max_rank - 1));
    let completed = current - 1;
    let variable_total = (GROWTH_MATERIAL_TOTAL - steps).max(0);
    let denominator = steps.saturating_mul(steps).max(1);
    let previous = variable_total
        .saturating_mul(completed)
        .saturating_mul(completed)
        / denominator;
    let next = variable_total
        .saturating_mul(current)
        .saturating_mul(current)
        / denominator;
    1 + next.saturating_sub(previous)
}

fn growth_gold_cost(current_rank: i32, base: i64, cube_multiplier: i64) -> i64 {
    let rank = i64::from(current_rank.max(1));
    base.saturating_add(
        rank.saturating_mul(rank)
            .saturating_mul(rank)
            .saturating_mul(cube_multiplier.max(0)),
    )
}

fn scaled_bonus(current: i64, total: i64, max: i64) -> i64 {
    total.saturating_mul(current.clamp(1, max)) / max.max(1)
}

async fn ensure_consumable(pool: &PgPool, template_id: &str) -> Result<(), SystemsActionError> {
    if template_id.is_empty() {
        return Err(SystemsActionError::NotFound);
    }
    let row: Option<(String,)> = sqlx::query_as(
        "select id from item_templates where id = $1 and kind = 'consumable'",
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await?;
    if row.is_none() {
        return Err(SystemsActionError::NotFound);
    }
    Ok(())
}

async fn lock_character(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
) -> Result<CharacterRecord, SystemsActionError> {
    sqlx::query_as::<_, CharacterRecord>(
        r#"
        select id, account_id, name, class, level, exp, gold, yuanbao, power
        from characters
        where id = $1 and deleted_at is null
        for update
        "#,
    )
    .bind(character_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(SystemsActionError::NotFound)
}

async fn refreshed_character(pool: &PgPool, character_id: i64) -> Result<CharacterRecord, sqlx::Error> {
    sqlx::query_as::<_, CharacterRecord>(
        r#"
        select id, account_id, name, class, level, exp, gold, yuanbao, power
        from characters
        where id = $1 and deleted_at is null
        "#,
    )
    .bind(character_id)
    .fetch_one(pool)
    .await
}

async fn debit_gold(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    gold: i64,
) -> Result<(), SystemsActionError> {
    let updated = sqlx::query("update characters set gold = gold - $2 where id = $1 and gold >= $2")
        .bind(character_id)
        .bind(gold.max(0))
        .execute(&mut **tx)
        .await?;
    if updated.rows_affected() == 0 {
        return Err(SystemsActionError::NotEnoughGold);
    }
    Ok(())
}

async fn consume_stack(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), SystemsActionError> {
    let quantity = quantity.max(1);
    let rows = sqlx::query_as::<_, MaterialStackRow>(
        r#"
        select id, quantity
        from inventory_items
        where character_id = $1 and template_id = $2 and location = 'bag'
        order by bind desc, id asc
        for update
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .fetch_all(&mut **tx)
    .await?;
    if rows.iter().map(|row| row.quantity).sum::<i64>() < quantity {
        return Err(SystemsActionError::NotEnoughMaterial);
    }

    let mut remaining = quantity;
    for row in rows {
        if remaining <= 0 {
            break;
        }
        if row.quantity <= remaining {
            sqlx::query("delete from inventory_items where id = $1")
                .bind(row.id)
                .execute(&mut **tx)
                .await?;
            remaining -= row.quantity;
        } else {
            sqlx::query("update inventory_items set quantity = quantity - $2 where id = $1")
                .bind(row.id)
                .bind(remaining)
                .execute(&mut **tx)
                .await?;
            remaining = 0;
        }
    }
    Ok(())
}

#[derive(Debug, FromRow)]
struct MaterialStackRow {
    id: i64,
    quantity: i64,
}
