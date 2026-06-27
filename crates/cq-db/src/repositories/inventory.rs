use cq_domain::{
    character::{exp_for_level, initial_stats, power_from_stats, Attributes, CharacterClass, CharacterStats},
    inventory::bag_limit_for_level,
};
use cq_protocol::dto::{
    InventoryActionResult, InventoryView, PlayerEquipmentSlots, PlayerInventoryItemView, PlayerInventorySummary,
    UseItemResult,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::Serialize;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;

use super::{
    character::{character_view, CharacterRecord},
    skill::{SkillBonus, SkillRepository},
    systems::{SystemBonus, SystemsRepository},
};

const INVENTORY_VIEW_BAG_LIMIT: i64 = 5_000;
const INVENTORY_VIEW_WAREHOUSE_LIMIT: i64 = 5_000;
const MANUAL_DECOMPOSE_BATCH_LIMIT: i64 = 5_000;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct InventoryItemRecord {
    pub id: i64,
    pub character_id: i64,
    pub template_id: String,
    pub quantity: i64,
    pub location: String,
    pub slot: Option<String>,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct EquipmentBonus {
    pub score: i64,
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
    pub dex: i64,
    pub spirit: i64,
    pub hp: i64,
    pub mp: i64,
    pub crit_pct: i64,
    pub life_steal_pct: i64,
    pub mana_steal_pct: i64,
    pub heavy_hit_pct: i64,
    pub paralyze_pct: i64,
    pub petrify_pct: i64,
    pub atk_pct: i64,
    pub def_pct: i64,
    pub hp_pct: i64,
    pub mp_pct: i64,
    pub skill_damage_pct: i64,
    pub boss_damage_pct: i64,
    pub damage_deepen_pct: i64,
    pub crit_damage_pct: i64,
    pub battle_end_restore_pct: i64,
    pub afk_base_reward_pct: i64,
    pub afk_exp_pct: i64,
    pub afk_gold_pct: i64,
    pub afk_drop_quality_pct: i64,
    pub afk_kill_pct: i64,
    pub afk_extra_minutes: i64,
    pub afk_offline_reward_pct: i64,
    pub all_skill_bonus: i64,
    pub normal_mob_execute_pct: i64,
    pub damage_reduce_pct: i64,
    pub paralyze_resist_pct: i64,
    pub petrify_resist_pct: i64,
    pub ignore_def_pct: i64,
    pub guaranteed_hit_pct: i64,
    pub target_max_hp_true_damage_pct: i64,
    pub self_max_mp_true_damage_pct: i64,
    pub creation_strike_pct: i64,
    pub creation_strike_damage_pct: i64,
    pub creation_strike_full_restore: bool,
    pub control_immune: bool,
    pub fatigue_immune: bool,
}

#[derive(Debug, Clone, FromRow)]
struct InventoryItemRow {
    id: i64,
    character_id: i64,
    template_id: String,
    name: String,
    kind: String,
    template_slot: Option<String>,
    rarity: String,
    price: i64,
    stackable: bool,
    stats: serde_json::Value,
    quantity: i64,
    location: String,
    equipped_slot: Option<String>,
    bind: bool,
    durability: i32,
    extra: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum InventoryActionError {
    #[error("item not found")]
    NotFound,
    #[error("not enough gold")]
    NotEnoughGold,
    #[error("not enough yuanbao")]
    NotEnoughYuanbao,
    #[error("bag is full")]
    BagFull,
    #[error("max enhance level reached")]
    MaxEnhance,
    #[error("not enough material: {0}")]
    NotEnoughMaterial(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, FromRow)]
struct EnhanceRow {
    name: String,
    slot: Option<String>,
    gold: i64,
    enhance_level: i32,
}

#[derive(Debug, FromRow)]
struct StackRow {
    id: i64,
    quantity: i64,
}

#[derive(Debug, FromRow)]
struct PowerStatsRow {
    str_: i64,
    dex: i64,
    int_: i64,
    con: i64,
    spirit: i64,
    max_hp: i64,
    max_mp: i64,
    atk: i64,
    def: i64,
    mag: i64,
    mdef: i64,
}

#[derive(Debug, FromRow)]
struct RecycleRow {
    name: String,
    price: i64,
    quantity: i64,
    enhance_level: i32,
}

#[derive(Debug, FromRow)]
struct DecomposeRow {
    id: i64,
    template_id: String,
    name: String,
    rarity: String,
    price: i64,
    quantity: i64,
    enhance_level: i32,
    tier: i64,
    slot: Option<String>,
    stats: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct MiscDecomposeRow {
    id: i64,
    template_id: String,
    name: String,
    kind: String,
    rarity: String,
    price: i64,
    quantity: i64,
}

#[derive(Debug, FromRow)]
struct ShopTemplateRow {
    name: String,
    price: i64,
    stackable: bool,
    stats: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct GrantTemplateRow {
    slot: Option<String>,
    price: i64,
    stackable: bool,
    stats: serde_json::Value,
}

#[derive(Debug, FromRow)]
struct VipAutoDecomposeRow {
    active_vip: bool,
    auto_decompose_enabled: bool,
    auto_decompose_max_tier: i32,
}

#[derive(Debug, FromRow)]
struct VipAutoExtractRow {
    active_vip: bool,
    wanxiang_unlocked: bool,
    auto_extract_essence_enabled: bool,
    auto_extract_essence_max_tier: i32,
}

#[derive(Debug, Clone, FromRow)]
struct ConsumableRow {
    id: i64,
    character_id: i64,
    name: String,
    kind: String,
    stats: serde_json::Value,
    quantity: i64,
    class: String,
    level: i32,
    exp: i64,
    max_hp: i64,
    max_mp: i64,
}

#[derive(Debug, Clone)]
pub struct ConsumedPotion {
    pub name: String,
    pub hp: i64,
    pub mp: i64,
    pub hp_pct: i64,
    pub mp_pct: i64,
    pub full_restore: bool,
}

#[derive(Debug, Clone, FromRow)]
struct AutoPotionRow {
    id: i64,
    name: String,
    stats: serde_json::Value,
    quantity: i64,
}

#[derive(Debug, Clone, FromRow)]
struct DropPenaltyRow {
    id: i64,
    name: String,
}

#[derive(Debug, Clone, FromRow)]
struct DropBagPenaltyRow {
    id: i64,
    name: String,
    quantity: i64,
}

#[derive(Debug, Clone)]
pub struct InventoryRepository {
    pool: PgPool,
}

impl InventoryRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn refresh_character_power(&self, character_id: i64) -> Result<CharacterRecord, sqlx::Error> {
        let base = self.power_stats(character_id).await?;
        let equipment = self.equipment_bonus(character_id).await?;
        let skills = SkillRepository::new(&self.pool).bonus(character_id).await?;
        let systems = SystemsRepository::new(&self.pool).combat_bonus(character_id).await?;
        let power = power_with_bonuses(&base, &equipment, &skills, &systems).max(0);
        sqlx::query_as::<_, CharacterRecord>(
            r#"
            update characters
            set power = $2
            where id = $1 and deleted_at is null
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(character_id)
        .bind(power)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn clamp_character_resources(&self, character_id: i64) -> Result<(), sqlx::Error> {
        let (max_hp, max_mp) = self.character_resource_caps(character_id).await?;
        sqlx::query(
            r#"
            update character_state
            set hp = least(greatest(hp, 0), $2),
                mp = least(greatest(mp, 0), $3)
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(max_hp.max(1))
        .bind(max_mp.max(0))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn character_resource_caps(&self, character_id: i64) -> Result<(i64, i64), sqlx::Error> {
        let base = self.power_stats(character_id).await?;
        let equipment = self.equipment_bonus(character_id).await?;
        let skills = SkillRepository::new(&self.pool).bonus(character_id).await?;
        let systems = SystemsRepository::new(&self.pool).combat_bonus(character_id).await?;
        let hp = base
            .max_hp
            .saturating_add(equipment.hp)
            .saturating_add(skills.hp)
            .saturating_add(systems.hp);
        let mp = base
            .max_mp
            .saturating_add(equipment.mp)
            .saturating_add(skills.mp)
            .saturating_add(systems.mp);
        Ok((
            apply_pct_i64(hp, equipment.hp_pct + systems.hp_pct),
            apply_pct_i64(mp, equipment.mp_pct + systems.mp_pct),
        ))
    }

    async fn power_stats(&self, character_id: i64) -> Result<CharacterStats, sqlx::Error> {
        let row = sqlx::query_as::<_, PowerStatsRow>(
            r#"
            select
              "str" as str_,
              dex,
              "int" as int_,
              con,
              spirit,
              max_hp,
              max_mp,
              atk,
              def,
              mag,
              mdef
            from character_stats
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(CharacterStats {
            character_id,
            attrs: Attributes {
                str_: row.str_,
                dex: row.dex,
                int_: row.int_,
                con: row.con,
                spirit: row.spirit,
            },
            max_hp: row.max_hp,
            max_mp: row.max_mp,
            atk: row.atk,
            def: row.def,
            mag: row.mag,
            mdef: row.mdef,
        })
    }

    pub async fn list_bag(&self, character_id: i64) -> Result<Vec<InventoryItemRecord>, sqlx::Error> {
        sqlx::query_as::<_, InventoryItemRecord>(
            r#"
            select id, character_id, template_id, quantity, location, slot
            from inventory_items
            where character_id = $1 and location in ('bag', 'equipped')
            order by id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn view(&self, character_id: i64, level: i32) -> Result<InventoryView, sqlx::Error> {
        let rows = self
            .item_rows(
                character_id,
                (bag_limit_for_level(level) as i64).min(INVENTORY_VIEW_BAG_LIMIT),
                INVENTORY_VIEW_WAREHOUSE_LIMIT,
            )
            .await?;
        let (bag_used, warehouse_used) = self.inventory_counts(character_id).await?;
        let summary = summarize_inventory(&rows, level, bag_used, warehouse_used);
        Ok(InventoryView {
            summary,
            items: rows.into_iter().map(item_view).collect(),
        })
    }

    pub async fn summary(&self, character_id: i64, level: i32) -> Result<PlayerInventorySummary, sqlx::Error> {
        let rows = self.item_rows(character_id, 0, 0).await?;
        let (bag_used, warehouse_used) = self.inventory_counts(character_id).await?;
        Ok(summarize_inventory(&rows, level, bag_used, warehouse_used))
    }

    pub async fn grant_item(
        &self,
        character_id: i64,
        template_id: &str,
        quantity: i64,
    ) -> Result<Option<PlayerInventoryItemView>, sqlx::Error> {
        self.grant_item_internal(character_id, template_id, quantity, true).await
    }

    pub async fn grant_item_direct(
        &self,
        character_id: i64,
        template_id: &str,
        quantity: i64,
    ) -> Result<Option<PlayerInventoryItemView>, sqlx::Error> {
        self.grant_item_internal(character_id, template_id, quantity, false).await
    }

    async fn grant_item_internal(
        &self,
        character_id: i64,
        template_id: &str,
        quantity: i64,
        apply_auto_grants: bool,
    ) -> Result<Option<PlayerInventoryItemView>, sqlx::Error> {
        let quantity = quantity.max(1);
        let template = sqlx::query_as::<_, GrantTemplateRow>(
            r#"
            select slot, price, stackable, stats
            from item_templates
            where id = $1
            "#,
        )
            .bind(template_id)
            .fetch_optional(&self.pool)
            .await?;
        let Some(template) = template else {
            return Ok(None);
        };
        let generated_extra = generated_equipment_extra(&template.stats, template.slot.as_deref());

        if apply_auto_grants
            && should_auto_extract_essence_grant(&self.pool, character_id, template_id, quantity, &template).await?
        {
            return Ok(None);
        }

        if apply_auto_grants && should_auto_decompose_grant(&self.pool, character_id, &template).await? {
            let reward = equipment_decompose_reward(
                template_id,
                template.price,
                quantity,
                0,
                stat_i64(&template.stats, "tier").max(1),
                template.slot.as_deref(),
                &template.stats,
            );
            let mut tx = self.pool.begin().await?;
            grant_equipment_decompose_rewards(&mut tx, character_id, &reward).await?;
            tx.commit().await?;
            return Ok(None);
        }

        let mut tx = self.pool.begin().await?;
        if template.stackable {
            if let Some(updated) = sqlx::query_as::<_, InventoryItemRow>(
                r#"
                update inventory_items
                set quantity = quantity + $3
                where id = (
                  select id
                  from inventory_items
                  where character_id = $1 and template_id = $2 and location = 'bag'
                  order by id asc
                  limit 1
                )
                returning id, character_id, template_id,
                  (select name from item_templates where id = inventory_items.template_id) as name,
                  (select kind from item_templates where id = inventory_items.template_id) as kind,
                  (select slot from item_templates where id = inventory_items.template_id) as template_slot,
                  (select rarity from item_templates where id = inventory_items.template_id) as rarity,
                  (select price from item_templates where id = inventory_items.template_id) as price,
                  (select stackable from item_templates where id = inventory_items.template_id) as stackable,
                  (select stats from item_templates where id = inventory_items.template_id) as stats,
                  quantity, location, slot as equipped_slot, bind, durability, extra
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity)
            .fetch_optional(&mut *tx)
            .await?
            {
                tx.commit().await?;
                return Ok(Some(item_view(updated)));
            }
        }

        let location = if bag_has_room_for_new_rows(&mut tx, character_id, 1).await? {
            "bag"
        } else if apply_auto_grants {
            return Ok(None);
        } else {
            "warehouse"
        };
        let inserted = sqlx::query_as::<_, InventoryItemRow>(
            r#"
            insert into inventory_items (character_id, template_id, quantity, location, extra)
            values ($1, $2, $3, $4, $5)
            returning id, character_id, template_id,
              (select name from item_templates where id = inventory_items.template_id) as name,
              (select kind from item_templates where id = inventory_items.template_id) as kind,
              (select slot from item_templates where id = inventory_items.template_id) as template_slot,
              (select rarity from item_templates where id = inventory_items.template_id) as rarity,
              (select price from item_templates where id = inventory_items.template_id) as price,
              (select stackable from item_templates where id = inventory_items.template_id) as stackable,
              (select stats from item_templates where id = inventory_items.template_id) as stats,
              quantity, location, slot as equipped_slot, bind, durability, extra
            "#,
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .bind(location)
        .bind(generated_extra)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(item_view(inserted)))
    }

    pub async fn enhance(&self, character_id: i64, item_id: i64) -> Result<InventoryActionResult, InventoryActionError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, EnhanceRow>(
            r#"
            select
              it.name,
              it.slot,
              c.gold,
              coalesce((ii.extra->>'enhance')::integer, 0) as enhance_level
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            join characters c on c.id = ii.character_id
            where ii.id = $1
              and ii.character_id = $2
              and ii.location in ('bag', 'equipped')
            for update of ii, c
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        if row.slot.is_none() {
            return Err(InventoryActionError::NotFound);
        }
        if row.enhance_level >= 20 {
            return Err(InventoryActionError::MaxEnhance);
        }
        let next_level = row.enhance_level + 1;
        let cost = i64::from(next_level).pow(2) * 100;
        if row.gold < cost {
            return Err(InventoryActionError::NotEnoughGold);
        }
        let materials = enhance_material_requirements(next_level);
        for material in &materials {
            let available = material_quantity(&mut tx, character_id, material.template_id).await?;
            if available < material.quantity {
                return Err(InventoryActionError::NotEnoughMaterial(format!(
                    "{}不足，需要 {} 个，当前 {} 个",
                    material.name, material.quantity, available
                )));
            }
        }

        sqlx::query("update characters set gold = gold - $2 where id = $1")
            .bind(character_id)
            .bind(cost)
            .execute(&mut *tx)
            .await?;
        for material in &materials {
            consume_material(&mut tx, character_id, material.template_id, material.quantity).await?;
        }
        let success_rate = enhance_success_rate(next_level);
        if thread_rng().gen::<f64>() > success_rate {
            let message = match enhance_failure_penalty(next_level) {
                EnhanceFailurePenalty::Keep => {
                    format!(
                        "{} 强化 +{} 失败，强化等级保持 +{}，消耗 {} 金币、{}。",
                        row.name,
                        next_level,
                        row.enhance_level,
                        cost,
                        format_materials(&materials)
                    )
                }
                EnhanceFailurePenalty::Downgrade => {
                    let downgraded = (row.enhance_level - 1).max(0);
                    sqlx::query(
                        r#"
                        update inventory_items
                        set extra = jsonb_set(extra, '{enhance}', to_jsonb($2::integer), true)
                        where id = $1 and character_id = $3
                        "#,
                    )
                    .bind(item_id)
                    .bind(downgraded)
                    .bind(character_id)
                    .execute(&mut *tx)
                    .await?;
                    format!(
                        "{} 强化 +{} 失败，强化等级降为 +{}，消耗 {} 金币、{}。",
                        row.name,
                        next_level,
                        downgraded,
                        cost,
                        format_materials(&materials)
                    )
                }
                EnhanceFailurePenalty::Destroy => {
                    sqlx::query("delete from inventory_items where id = $1 and character_id = $2")
                        .bind(item_id)
                        .bind(character_id)
                        .execute(&mut *tx)
                        .await?;
                    format!(
                        "{} 强化 +{} 失败，装备碎裂销毁，消耗 {} 金币、{}。",
                        row.name,
                        next_level,
                        cost,
                        format_materials(&materials)
                    )
                }
            };
            tx.commit().await?;
            let character = self.refresh_character_power(character_id).await?;
            self.clamp_character_resources(character_id).await?;

            return Ok(InventoryActionResult {
                inventory: self.view_for_character(character_id).await?,
                character: character_view(character),
                message,
            });
        }
        sqlx::query(
            r#"
            update inventory_items
            set extra = jsonb_set(extra, '{enhance}', to_jsonb($2::integer), true)
            where id = $1 and character_id = $3
            "#,
        )
        .bind(item_id)
        .bind(next_level)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        let character = self.refresh_character_power(character_id).await?;
        self.clamp_character_resources(character_id).await?;

        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(character),
            message: format!("{} 强化到 +{}，消耗 {} 金币、{}。", row.name, next_level, cost, format_materials(&materials)),
        })
    }

    pub async fn recycle(&self, character_id: i64, item_id: i64) -> Result<InventoryActionResult, InventoryActionError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, RecycleRow>(
            r#"
            select
              it.name,
              it.price,
              ii.quantity,
              coalesce((ii.extra->>'enhance')::integer, 0) as enhance_level
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.id = $1
              and ii.character_id = $2
              and ii.location in ('bag', 'warehouse')
              and not exists (select 1 from consignments c where c.item_id = ii.id)
            for update of ii
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let gold = (row.price * row.quantity / 5 + i64::from(row.enhance_level) * 50).max(1);
        sqlx::query("delete from inventory_items where id = $1 and character_id = $2")
            .bind(item_id)
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("update characters set gold = gold + $2 where id = $1")
            .bind(character_id)
            .bind(gold)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!("回收 {}，获得 {} 金币。", row.name, gold),
        })
    }

    pub async fn decompose(
        &self,
        character_id: i64,
        rarities: &[String],
        item_ids: &[i64],
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let rarity_filter = rarities.to_vec();
        let item_filter = item_ids.to_vec();
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query_as::<_, DecomposeRow>(
            r#"
            select
              ii.id,
              ii.template_id,
              it.name,
              it.rarity,
              it.price,
              ii.quantity,
              case
                when coalesce(ii.extra->>'enhance', '') ~ '^[0-9]+$'
                  then (ii.extra->>'enhance')::integer
                else 0
              end as enhance_level,
              greatest(
                case
                  when coalesce(it.stats->>'tier', '') ~ '^[0-9]+$'
                    then (it.stats->>'tier')::bigint
                  else 1
                end,
                1
              ) as tier,
              it.slot,
              it.stats
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location in ('bag', 'warehouse')
              and ii.bind = false
              and (it.slot is not null or lower(coalesce(it.stats->>'decompose_only', 'false')) = 'true')
              and (coalesce(array_length($2::text[], 1), 0) = 0 or it.rarity = any($2::text[]))
              and (coalesce(array_length($3::bigint[], 1), 0) = 0 or ii.id = any($3::bigint[]))
              and not exists (select 1 from consignments c where c.item_id = ii.id)
            order by ii.id asc
            limit $4
            "#,
        )
        .bind(character_id)
        .bind(&rarity_filter)
        .bind(&item_filter)
        .bind(MANUAL_DECOMPOSE_BATCH_LIMIT)
        .fetch_all(&mut *tx)
        .await?;
        if rows.is_empty() {
            return Err(InventoryActionError::NotFound);
        }

        let reward = rows.iter().fold(EquipmentDecomposeReward::default(), |mut acc, row| {
            acc.add(equipment_decompose_reward(
                &row.template_id,
                row.price,
                row.quantity,
                row.enhance_level,
                row.tier,
                row.slot.as_deref(),
                &row.stats,
            ));
            acc
        });
        let names = rows
            .iter()
            .take(3)
            .map(|row| row.name.clone())
            .collect::<Vec<_>>()
            .join("、");
        let ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
        sqlx::query("delete from inventory_items where character_id = $1 and id = any($2::bigint[])")
            .bind(character_id)
            .bind(&ids)
            .execute(&mut *tx)
            .await?;
        grant_equipment_decompose_rewards(&mut tx, character_id, &reward).await?;
        tx.commit().await?;

        let suffix = if rows.len() > 3 { "等装备" } else { "" };
        let batch_note = if rows.len() as i64 >= MANUAL_DECOMPOSE_BATCH_LIMIT && item_filter.is_empty() {
            "本次已处理 5000 件，背包仍超量时可继续点击一键拆解。"
        } else {
            ""
        };
        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!(
                "拆解 {}{}，获得炼器石 {} 个、鸿蒙石 {} 个、金币 {}、元宝 {}。{}",
                names,
                suffix,
                reward.refine_stones,
                reward.hongmeng_stones,
                reward.gold,
                reward.yuanbao,
                batch_note
            ),
        })
    }

    pub async fn decompose_misc(
        &self,
        character_id: i64,
        kinds: &[String],
        item_ids: &[i64],
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let kind_filter = if kinds.is_empty() {
            vec!["book".to_string()]
        } else {
            kinds.to_vec()
        };
        let item_filter = item_ids.to_vec();
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query_as::<_, MiscDecomposeRow>(
            r#"
            select
              ii.id,
              ii.template_id,
              it.name,
              it.kind,
              it.rarity,
              it.price,
              ii.quantity
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location in ('bag', 'warehouse')
              and it.slot is null
              and it.kind = any($2::text[])
              and (cardinality($3::bigint[]) = 0 or ii.id = any($3::bigint[]))
              and not exists (select 1 from consignments c where c.item_id = ii.id)
            for update of ii
            "#,
        )
        .bind(character_id)
        .bind(&kind_filter)
        .bind(&item_filter)
        .fetch_all(&mut *tx)
        .await?;
        if rows.is_empty() {
            return Err(InventoryActionError::NotFound);
        }

        let ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
        let mut skill_pages = 0_i64;
        let mut insight_pills = 0_i64;
        let mut gold = 0_i64;
        for row in &rows {
            let (pages, insights, row_gold) = misc_decompose_reward(row);
            skill_pages += pages;
            insight_pills += insights;
            gold += row_gold;
        }
        let names = rows
            .iter()
            .take(3)
            .map(|row| row.name.clone())
            .collect::<Vec<_>>()
            .join("、");

        sqlx::query("delete from inventory_items where character_id = $1 and id = any($2::bigint[])")
            .bind(character_id)
            .bind(&ids)
            .execute(&mut *tx)
            .await?;
        if gold > 0 {
            sqlx::query("update characters set gold = gold + $2 where id = $1")
                .bind(character_id)
                .bind(gold)
                .execute(&mut *tx)
                .await?;
        }
        grant_stackable(&mut tx, character_id, "skill_page", skill_pages).await?;
        grant_stackable(&mut tx, character_id, "pill_insight", insight_pills).await?;
        tx.commit().await?;

        let suffix = if rows.len() > 3 { "等杂项" } else { "" };
        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!(
                "拆解 {}{}，获得技能书残页 {} 个、悟性丹 {} 个、金币 {}。",
                names,
                suffix,
                skill_pages,
                insight_pills,
                gold
            ),
        })
    }

    pub async fn exchange_insight_material(
        &self,
        character_id: i64,
        material_id: &str,
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let (template_id, name) = match material_id.trim() {
            "treasure_shard" => ("treasure_shard", "法宝碎片"),
            "cultivation_pill" => ("cultivation_pill", "修炼丹"),
            "pet_food" => ("pet_food", "灵兽粮"),
            _ => return Err(InventoryActionError::NotFound),
        };
        let mut tx = self.pool.begin().await?;
        consume_stackable(&mut tx, character_id, "pill_insight", 10).await?;
        grant_stackable(&mut tx, character_id, template_id, 1).await?;
        tx.commit().await?;
        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!("商人协会兑换成功：悟性丹 x10 换得 {} x1。", name),
        })
    }

    pub async fn upgrade_battle_instinct(
        &self,
        character_id: i64,
    ) -> Result<InventoryActionResult, InventoryActionError> {
        self.upgrade_special_passive(character_id, "talent_battle_instinct").await
    }

    pub async fn upgrade_special_passive(
        &self,
        character_id: i64,
        skill_id: &str,
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let skill_id = skill_id.trim();
        let mut tx = self.pool.begin().await?;
        let (skill_name,): (String,) = sqlx::query_as(
            r#"
            select name
            from skills
            where id = $1
              and coalesce((config->>'special_upgrade_only')::boolean, false) = true
            "#,
        )
        .bind(skill_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let current_level = sqlx::query_as::<_, (Option<i32>,)>(
            "select max(level) from character_skills where character_id = $1 and skill_id = $2",
        )
        .bind(character_id)
        .bind(skill_id)
        .fetch_one(&mut *tx)
        .await?
        .0
        .unwrap_or_default()
        .clamp(0, 100);
        if current_level >= 100 {
            return Err(InventoryActionError::MaxEnhance);
        }
        let cost = i64::from(current_level.max(1)).saturating_mul(500);
        consume_stackable(&mut tx, character_id, "skill_page", cost).await?;
        sqlx::query(
            r#"
            insert into character_skills (character_id, skill_id, level, proficiency)
            values ($1, $2, $3, 0)
            on conflict (character_id, skill_id) do update set
              level = greatest(character_skills.level, excluded.level),
              proficiency = character_skills.proficiency
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .bind(current_level + 1)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!(
                "不动冥王参悟成功：{} {} -> {}，消耗技能书残页 {}。",
                skill_name,
                current_level,
                current_level + 1,
                cost
            ),
        })
    }

    pub async fn store(&self, character_id: i64, item_id: i64) -> Result<InventoryActionResult, InventoryActionError> {
        let updated = sqlx::query(
            r#"
            update inventory_items
            set location = 'warehouse', slot = null
            where id = $1 and character_id = $2 and location = 'bag'
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(InventoryActionError::NotFound);
        }
        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: "物品已存入仓库。".into(),
        })
    }

    pub async fn retrieve(&self, character_id: i64, item_id: i64) -> Result<InventoryActionResult, InventoryActionError> {
        let level: (i32,) = sqlx::query_as("select level from characters where id = $1 and deleted_at is null")
            .bind(character_id)
            .fetch_one(&self.pool)
            .await?;
        let (bag_count,): (i64,) =
            sqlx::query_as("select count(*)::bigint from inventory_items where character_id = $1 and location = 'bag'")
                .bind(character_id)
                .fetch_one(&self.pool)
                .await?;
        if bag_count as usize >= bag_limit_for_level(level.0) {
            return Err(InventoryActionError::BagFull);
        }
        let updated = sqlx::query(
            r#"
            update inventory_items
            set location = 'bag', slot = null
            where id = $1 and character_id = $2 and location = 'warehouse'
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(InventoryActionError::NotFound);
        }
        Ok(InventoryActionResult {
            inventory: self.view(character_id, level.0).await?,
            character: character_view(self.character(character_id).await?),
            message: "物品已取回背包。".into(),
        })
    }

    pub async fn buy_template(
        &self,
        character_id: i64,
        template_id: &str,
        quantity: i64,
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let quantity = quantity.clamp(1, 999);
        let mut tx = self.pool.begin().await?;
        let template = sqlx::query_as::<_, ShopTemplateRow>(
            "select name, price, stackable, stats from item_templates where id = $1 and price > 0 and flags->>'shop' = 'supply'",
        )
        .bind(template_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let character = sqlx::query_as::<_, CharacterRecord>(
            "select id, account_id, name, class, level, exp, gold, yuanbao, power from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let cost = template.price.saturating_mul(quantity);
        if character.gold < cost {
            return Err(InventoryActionError::NotEnoughGold);
        }
        let (bag_count,): (i64,) =
            sqlx::query_as("select count(*)::bigint from inventory_items where character_id = $1 and location = 'bag'")
                .bind(character_id)
                .fetch_one(&mut *tx)
                .await?;
        let existing_stack = if template.stackable {
            let (exists,): (bool,) = sqlx::query_as(
                "select exists(select 1 from inventory_items where character_id = $1 and template_id = $2 and location = 'bag')",
            )
            .bind(character_id)
            .bind(template_id)
            .fetch_one(&mut *tx)
            .await?;
            exists
        } else {
            false
        };
        let rows_needed = if template.stackable {
            if existing_stack { 0 } else { 1 }
        } else {
            quantity
        };
        if (bag_count.saturating_add(rows_needed) as usize) > bag_limit_for_level(character.level) {
            return Err(InventoryActionError::BagFull);
        }
        sqlx::query("update characters set gold = gold - $2 where id = $1")
            .bind(character_id)
            .bind(cost)
            .execute(&mut *tx)
            .await?;
        if template.stackable {
            let updated = sqlx::query(
                r#"
                update inventory_items
                set quantity = quantity + $3
                where id = (
                  select id from inventory_items
                  where character_id = $1 and template_id = $2 and location = 'bag'
                  order by id asc
                  limit 1
                )
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() == 0 {
                sqlx::query(
                    "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, $3, 'bag')",
                )
                .bind(character_id)
                .bind(template_id)
                .bind(quantity)
                .execute(&mut *tx)
                .await?;
            }
        } else {
            for _ in 0..quantity {
                sqlx::query(
                    "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, 1, 'bag')",
                )
                .bind(character_id)
                .bind(template_id)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;

        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!("购买 {} x{}，消耗 {} 金币。", template.name, quantity, cost),
        })
    }

    pub async fn buy_yuanbao_template(
        &self,
        character_id: i64,
        template_id: &str,
        quantity: i64,
    ) -> Result<InventoryActionResult, InventoryActionError> {
        let quantity = quantity.clamp(1, 99);
        let mut tx = self.pool.begin().await?;
        let template = sqlx::query_as::<_, ShopTemplateRow>(
            r#"
            select
              name,
              coalesce((flags->>'yuanbao_price')::bigint, price) as price,
              stackable,
              stats
            from item_templates
            where id = $1
              and flags->>'shop' = 'yuanbao'
              and coalesce((flags->>'yuanbao_price')::bigint, price) > 0
            "#,
        )
        .bind(template_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let character = sqlx::query_as::<_, CharacterRecord>(
            "select id, account_id, name, class, level, exp, gold, yuanbao, power from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(InventoryActionError::NotFound)?;
        let cost = template.price.saturating_mul(quantity);
        if character.yuanbao < cost {
            return Err(InventoryActionError::NotEnoughYuanbao);
        }
        if stat_i64(&template.stats, "vip_days") > 0 {
            let days = stat_i64(&template.stats, "vip_days").clamp(1, 365);
            let total_days = days.saturating_mul(quantity);
            sqlx::query("update characters set yuanbao = yuanbao - $2 where id = $1")
                .bind(character_id)
                .bind(cost)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                r#"
                insert into vip_records (character_id, tier, starts_at, ends_at)
                values (
                  $1,
                  'vip',
                  now(),
                  greatest(
                    now(),
                    coalesce(
                      (
                        select max(ends_at)
                        from vip_records
                        where character_id = $1
                          and tier = 'vip'
                          and ends_at is not null
                          and ends_at > now()
                      ),
                      now()
                    )
                  ) + ($2::int * interval '1 day')
                )
                "#,
            )
            .bind(character_id)
            .bind(total_days as i32)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(InventoryActionResult {
                inventory: self.view_for_character(character_id).await?,
                character: character_view(self.character(character_id).await?),
                message: format!("购买 {} x{}，激活会员 {} 天，消耗 {} 元宝。", template.name, quantity, total_days, cost),
            });
        }
        let (bag_count,): (i64,) =
            sqlx::query_as("select count(*)::bigint from inventory_items where character_id = $1 and location = 'bag'")
                .bind(character_id)
                .fetch_one(&mut *tx)
                .await?;
        let existing_stack = if template.stackable {
            let (exists,): (bool,) = sqlx::query_as(
                "select exists(select 1 from inventory_items where character_id = $1 and template_id = $2 and location = 'bag')",
            )
            .bind(character_id)
            .bind(template_id)
            .fetch_one(&mut *tx)
            .await?;
            exists
        } else {
            false
        };
        let rows_needed = if template.stackable {
            if existing_stack { 0 } else { 1 }
        } else {
            quantity
        };
        if (bag_count.saturating_add(rows_needed) as usize) > bag_limit_for_level(character.level) {
            return Err(InventoryActionError::BagFull);
        }
        sqlx::query("update characters set yuanbao = yuanbao - $2 where id = $1")
            .bind(character_id)
            .bind(cost)
            .execute(&mut *tx)
            .await?;
        if template.stackable {
            let updated = sqlx::query(
                r#"
                update inventory_items
                set quantity = quantity + $3
                where id = (
                  select id from inventory_items
                  where character_id = $1 and template_id = $2 and location = 'bag'
                  order by id asc
                  limit 1
                )
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() == 0 {
                sqlx::query(
                    "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, $3, 'bag')",
                )
                .bind(character_id)
                .bind(template_id)
                .bind(quantity)
                .execute(&mut *tx)
                .await?;
            }
        } else {
            for _ in 0..quantity {
                sqlx::query(
                    "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, 1, 'bag')",
                )
                .bind(character_id)
                .bind(template_id)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;

        Ok(InventoryActionResult {
            inventory: self.view_for_character(character_id).await?,
            character: character_view(self.character(character_id).await?),
            message: format!("购买 {} x{}，消耗 {} 元宝。", template.name, quantity, cost),
        })
    }

    pub async fn equip(&self, character_id: i64, item_id: i64) -> Result<InventoryView, sqlx::Error> {
        let item: (Option<String>,) = sqlx::query_as(
            r#"
            select it.slot
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.id = $1 and ii.character_id = $2 and ii.location in ('bag', 'equipped')
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;

        let mut tx = self.pool.begin().await?;
        let template_slot = item.0.ok_or(sqlx::Error::RowNotFound)?;
        let slot = resolve_equip_slot(&mut tx, character_id, item_id, &template_slot).await?;
        sqlx::query(
            r#"
            update inventory_items
            set location = 'bag', slot = null
            where character_id = $1 and location = 'equipped' and slot = $2 and id <> $3
            "#,
        )
        .bind(character_id)
        .bind(&slot)
        .bind(item_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            update inventory_items
            set location = 'equipped', slot = $3
            where id = $1 and character_id = $2
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .bind(&slot)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        self.refresh_character_power(character_id).await?;
        self.clamp_character_resources(character_id).await?;
        self.view_for_character(character_id).await
    }

    pub async fn unequip(&self, character_id: i64, item_id: i64) -> Result<InventoryView, sqlx::Error> {
        sqlx::query(
            r#"
            update inventory_items
            set location = 'bag', slot = null
            where id = $1 and character_id = $2 and location = 'equipped'
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        self.refresh_character_power(character_id).await?;
        self.clamp_character_resources(character_id).await?;
        self.view_for_character(character_id).await
    }

    pub async fn use_item(
        &self,
        character_id: i64,
        item_id: i64,
        max_hp_override: Option<i64>,
        max_mp_override: Option<i64>,
    ) -> Result<UseItemResult, sqlx::Error> {
        let row = sqlx::query_as::<_, ConsumableRow>(
            r#"
            select
              ii.id,
              ii.character_id,
              it.name,
              it.kind,
              it.stats,
              ii.quantity,
              c.class,
              c.level,
              c.exp,
              (cs.max_hp + coalesce(bonus.hp, 0))::bigint as max_hp,
              (cs.max_mp + coalesce(bonus.mp, 0))::bigint as max_mp
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            join characters c on c.id = ii.character_id
            join character_stats cs on cs.character_id = ii.character_id
            left join lateral (
              select
                coalesce(sum(coalesce((eit.stats->>'hp')::bigint, 0)), 0)::bigint as hp,
                coalesce(sum(coalesce((eit.stats->>'mp')::bigint, 0)), 0)::bigint as mp
              from inventory_items eii
              join item_templates eit on eit.id = eii.template_id
              where eii.character_id = ii.character_id and eii.location = 'equipped'
            ) bonus on true
            where ii.id = $1 and ii.character_id = $2 and ii.location = 'bag'
            "#,
        )
        .bind(item_id)
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;

        if row.kind != "consumable" {
            return Err(sqlx::Error::RowNotFound);
        }

        let hp = stat_i64(&row.stats, "hp");
        let mp = stat_i64(&row.stats, "mp");
        let hp_pct = stat_i64(&row.stats, "hp_pct");
        let mp_pct = stat_i64(&row.stats, "mp_pct");
        let full_restore = row
            .stats
            .get("full_restore")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let exp = stat_i64(&row.stats, "exp");
        let skill_proficiency = stat_i64(&row.stats, "skill_proficiency");
        let teleport = row
            .stats
            .get("teleport")
            .and_then(|value| value.as_object())
            .and_then(|object| {
                let zone = object.get("zone")?.as_str()?;
                let room = object.get("room")?.as_str()?;
                Some((zone.to_owned(), room.to_owned()))
            });

        if hp == 0
            && mp == 0
            && hp_pct == 0
            && mp_pct == 0
            && !full_restore
            && exp == 0
            && skill_proficiency == 0
            && teleport.is_none()
        {
            return Err(sqlx::Error::RowNotFound);
        }

        let max_hp = max_hp_override.unwrap_or(row.max_hp).max(1);
        let max_mp = max_mp_override.unwrap_or(row.max_mp).max(0);
        let hp_restore = if full_restore { max_hp } else { hp + max_hp.saturating_mul(hp_pct) / 100 };
        let mp_restore = if full_restore { max_mp } else { mp + max_mp.saturating_mul(mp_pct) / 100 };

        let mut tx = self.pool.begin().await?;
        let mut next_level = row.level;
        if exp > 0 {
            let next_exp = row.exp.saturating_add(exp);
            while next_level < 500 && next_exp >= exp_for_level(next_level + 1) {
                next_level += 1;
            }
            let stats = initial_stats(class_from_str(&row.class), next_level);
            let power = power_from_stats(&stats);
            sqlx::query(
                r#"
                update characters
                set exp = $2, level = $3, power = $4
                where id = $1
                "#,
            )
            .bind(row.character_id)
            .bind(next_exp)
            .bind(next_level)
            .bind(power)
            .execute(&mut *tx)
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
            .bind(row.character_id)
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
            .execute(&mut *tx)
            .await?;
        }
        if skill_proficiency > 0 {
            sqlx::query(
                r#"
                update character_skills
                set proficiency = proficiency + $2,
                    level = case
                      when level < 10 then least(10, greatest(level, ((proficiency + $2) / 100)::integer + 1))
                      else level
                    end
                where character_id = $1
                "#,
            )
            .bind(row.character_id)
            .bind(skill_proficiency)
            .execute(&mut *tx)
            .await?;
        }
        if hp_restore != 0 || mp_restore != 0 || full_restore {
            sqlx::query(
                r#"
                update character_state
                set hp = least($2, case when $6 then $2 else hp + $4 end),
                    mp = least($3, case when $6 then $3 else mp + $5 end),
                    last_idle_regen_at = now()
                where character_id = $1
                "#,
            )
            .bind(row.character_id)
            .bind(max_hp)
            .bind(max_mp)
            .bind(hp_restore)
            .bind(mp_restore)
            .bind(full_restore)
            .execute(&mut *tx)
            .await?;
        }
        if let Some((zone, room)) = &teleport {
            sqlx::query(
                r#"
                update character_state
                set zone = $2,
                    room = $3,
                    last_idle_regen_at = now()
                where character_id = $1
                "#,
            )
            .bind(row.character_id)
            .bind(zone)
            .bind(room)
            .execute(&mut *tx)
            .await?;
        }
        if row.quantity > 1 {
            sqlx::query("update inventory_items set quantity = quantity - 1 where id = $1")
                .bind(row.id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("delete from inventory_items where id = $1")
                .bind(row.id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        self.refresh_character_power(row.character_id).await?;
        self.clamp_character_resources(row.character_id).await?;

        let mut effects = Vec::new();
        if full_restore {
            effects.push("恢复全部生命和魔法".into());
        } else if hp_restore > 0 {
            effects.push(format!("恢复 {} HP", hp_restore));
        }
        if !full_restore && mp_restore > 0 {
            effects.push(format!("恢复 {} 魔法", mp_restore));
        }
        if exp > 0 {
            effects.push(format!("获得 {} 修炼经验", exp));
        }
        if skill_proficiency > 0 {
            effects.push(format!("已学技能熟练度 +{}", skill_proficiency));
        }
        if teleport.is_some() {
            effects.push("回到安全区".into());
        }
        Ok(UseItemResult {
            inventory: self.view(row.character_id, next_level).await?,
            message: format!("使用 {}：{}。", row.name, effects.join("，")),
        })
    }

    pub async fn consume_auto_potion(
        &self,
        character_id: i64,
        template_id: &str,
    ) -> Result<Option<ConsumedPotion>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, AutoPotionRow>(
            r#"
            select ii.id, it.name, it.stats, ii.quantity
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.template_id = $2
              and ii.location = 'bag'
              and it.kind = 'consumable'
            order by ii.bind desc, ii.id asc
            limit 1
            for update
            "#,
        )
        .bind(character_id)
        .bind(template_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.commit().await?;
            return Ok(None);
        };

        if row.quantity > 1 {
            sqlx::query("update inventory_items set quantity = quantity - 1 where id = $1")
                .bind(row.id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("delete from inventory_items where id = $1")
                .bind(row.id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;

        Ok(Some(ConsumedPotion {
            name: row.name,
            hp: stat_i64(&row.stats, "hp"),
            mp: stat_i64(&row.stats, "mp"),
            hp_pct: stat_i64(&row.stats, "hp_pct"),
            mp_pct: stat_i64(&row.stats, "mp_pct"),
            full_restore: row
                .stats
                .get("full_restore")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }))
    }

    pub async fn drop_random_equipment(&self, character_id: i64) -> Result<Option<String>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DropPenaltyRow>(
            r#"
            select ii.id, it.name
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location in ('bag', 'equipped')
              and it.slot is not null
            order by ii.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let row = {
            let mut rng = thread_rng();
            rows[rng.gen_range(0..rows.len())].clone()
        };
        sqlx::query("delete from inventory_items where character_id = $1 and id = $2")
            .bind(character_id)
            .bind(row.id)
            .execute(&self.pool)
            .await?;
        self.refresh_character_power(character_id).await?;
        self.clamp_character_resources(character_id).await?;
        Ok(Some(row.name))
    }

    pub async fn drop_random_bag_items(&self, character_id: i64) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DropBagPenaltyRow>(
            r#"
            select ii.id, it.name, ii.quantity
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location = 'bag'
            order by ii.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        if rows.is_empty() {
            return Ok(vec![]);
        }
        let targets = {
            let mut rng = thread_rng();
            let mut shuffled = rows;
            shuffled.shuffle(&mut rng);
            let count = rng.gen_range(1..=shuffled.len().min(3));
            shuffled.into_iter().take(count).collect::<Vec<_>>()
        };
        for row in &targets {
            let item_id = row.id;
            sqlx::query("delete from inventory_items where character_id = $1 and id = $2")
                .bind(character_id)
                .bind(item_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(targets
            .into_iter()
            .map(|row| {
                if row.quantity > 1 {
                    format!("{} x{}", row.name, row.quantity)
                } else {
                    row.name
                }
            })
            .collect())
    }

    pub async fn drop_random_equipped_item(&self, character_id: i64) -> Result<Option<String>, sqlx::Error> {
        let (protected,): (bool,) = sqlx::query_as(
            r#"
            select exists(
              select 1
              from inventory_items ii
              join item_templates it on it.id = ii.template_id
              where ii.character_id = $1
                and ii.location = 'equipped'
                and coalesce((it.stats->>'death_drop_immune')::boolean, false)
            )
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        if protected {
            return Ok(None);
        }
        let rows = sqlx::query_as::<_, DropPenaltyRow>(
            r#"
            select ii.id, it.name
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location = 'equipped'
              and it.slot is not null
            order by ii.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let row = {
            let mut rng = thread_rng();
            rows[rng.gen_range(0..rows.len())].clone()
        };
        sqlx::query("delete from inventory_items where character_id = $1 and id = $2")
            .bind(character_id)
            .bind(row.id)
            .execute(&self.pool)
            .await?;
        self.refresh_character_power(character_id).await?;
        self.clamp_character_resources(character_id).await?;
        Ok(Some(row.name))
    }

    pub async fn trigger_origin_revive(&self, character_id: i64) -> Result<bool, sqlx::Error> {
        let cooldown_seconds: i32 = sqlx::query_scalar(
            r#"
            select coalesce(max(coalesce((it.stats->>'origin_revive_cd_seconds')::int, 0)), 0)::int
            from inventory_items ii
            join item_templates it on it.id = ii.template_id
            where ii.character_id = $1
              and ii.location = 'equipped'
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        if cooldown_seconds <= 0 {
            return Ok(false);
        }

        let triggered: Option<(bool,)> = sqlx::query_as(
            r#"
            insert into character_mechanism_cooldowns (character_id, mechanism, ready_at, updated_at)
            values ($1, 'origin_revive', now() + ($2::text || ' seconds')::interval, now())
            on conflict (character_id, mechanism) do update
            set ready_at = excluded.ready_at,
                updated_at = now()
            where character_mechanism_cooldowns.ready_at <= now()
            returning true
            "#,
        )
        .bind(character_id)
        .bind(cooldown_seconds)
        .fetch_optional(&self.pool)
        .await?;
        Ok(triggered.is_some())
    }

    pub async fn equipment_bonus(&self, character_id: i64) -> Result<EquipmentBonus, sqlx::Error> {
        sqlx::query_as::<_, EquipmentBonus>(
            r#"
            with equipped as (
              select *, coalesce((extra->>'enhance')::bigint, 0) as enhance
              from inventory_items
              where character_id = $1 and location = 'equipped'
            ),
            effective_equipped as (
              select
                e.*,
                it.kind,
                it.slot as template_slot,
                (it.stats || coalesce(e.extra->'generated_stats', '{}'::jsonb)) as stats,
                it.flags
              from equipped e
              join item_templates it on it.id = e.template_id
            ),
            base_bonus as (
              select
                coalesce(sum(coalesce((it.stats->>'score')::bigint, 0) + it.enhance * 20), 0)::bigint as score,
                coalesce(sum(coalesce((it.stats->>'atk')::bigint, 0) + case when it.kind = 'weapon' then it.enhance * 4 when it.kind = 'accessory' then it.enhance else 0 end), 0)::bigint as atk,
                coalesce(sum(coalesce((it.stats->>'def')::bigint, 0) + case when it.kind = 'armor' then it.enhance * 2 when it.kind = 'accessory' then it.enhance else 0 end), 0)::bigint as def,
                coalesce(sum(coalesce((it.stats->>'mag')::bigint, 0) + case when it.kind = 'accessory' then it.enhance else 0 end), 0)::bigint as mag,
                coalesce(sum(coalesce((it.stats->>'mdef')::bigint, 0) + case when it.kind = 'armor' and it.template_slot = 'chest' then it.enhance else 0 end), 0)::bigint as mdef,
                coalesce(sum(coalesce((it.stats->>'luck')::bigint, 0) + coalesce((it.stats->>'dex')::bigint, 0) + case when it.kind = 'accessory' then it.enhance else 0 end), 0)::bigint as dex,
                coalesce(sum(coalesce((it.stats->>'spirit')::bigint, 0)), 0)::bigint as spirit,
                coalesce(sum(coalesce((it.stats->>'hp')::bigint, 0)), 0)::bigint as hp,
                coalesce(sum(coalesce((it.stats->>'mp')::bigint, 0)), 0)::bigint as mp,
                coalesce(sum(coalesce((it.stats->>'crit_pct')::bigint, 0) + coalesce((it.stats->>'crit')::bigint, 0)), 0)::bigint as crit_pct,
                coalesce(sum(
                  coalesce((it.stats->>'life_steal_pct')::bigint, 0)
                  + case when it.kind = 'weapon' and it.enhance >= 11 then it.enhance - 10 else 0 end
                ), 0)::bigint as life_steal_pct,
                coalesce(sum(coalesce((it.stats->>'mana_steal_pct')::bigint, 0)), 0)::bigint as mana_steal_pct,
                coalesce(sum(coalesce((it.stats->>'heavy_hit_pct')::bigint, 0)), 0)::bigint as heavy_hit_pct,
                coalesce(sum(coalesce((it.stats->>'paralyze_pct')::bigint, 0)), 0)::bigint as paralyze_pct,
                coalesce(sum(coalesce((it.stats->>'petrify_pct')::bigint, 0)), 0)::bigint as petrify_pct,
                coalesce(sum(coalesce((it.stats->>'atk_pct')::bigint, 0)), 0)::bigint as atk_pct,
                coalesce(sum(coalesce((it.stats->>'def_pct')::bigint, 0)), 0)::bigint as def_pct,
                coalesce(sum(coalesce((it.stats->>'hp_pct')::bigint, 0)), 0)::bigint as hp_pct,
                coalesce(sum(coalesce((it.stats->>'mp_pct')::bigint, 0)), 0)::bigint as mp_pct,
                coalesce(sum(coalesce((it.stats->>'skill_damage_pct')::bigint, 0)), 0)::bigint as skill_damage_pct,
                coalesce(sum(coalesce((it.stats->>'boss_damage_pct')::bigint, 0)), 0)::bigint as boss_damage_pct,
                coalesce(sum(coalesce((it.stats->>'damage_deepen_pct')::bigint, 0)), 0)::bigint as damage_deepen_pct,
                coalesce(sum(coalesce((it.stats->>'crit_damage_pct')::bigint, 0)), 0)::bigint as crit_damage_pct,
                coalesce(sum(coalesce((it.stats->>'battle_end_restore_pct')::bigint, 0)), 0)::bigint as battle_end_restore_pct,
                coalesce(sum(coalesce((it.stats->>'afk_base_reward_pct')::bigint, 0)), 0)::bigint as afk_base_reward_pct,
                coalesce(sum(coalesce((it.stats->>'afk_exp_pct')::bigint, 0)), 0)::bigint as afk_exp_pct,
                coalesce(sum(coalesce((it.stats->>'afk_gold_pct')::bigint, 0)), 0)::bigint as afk_gold_pct,
                coalesce(sum(coalesce((it.stats->>'afk_drop_quality_pct')::bigint, 0)), 0)::bigint as afk_drop_quality_pct,
                coalesce(sum(coalesce((it.stats->>'afk_kill_pct')::bigint, 0)), 0)::bigint as afk_kill_pct,
                coalesce(sum(coalesce((it.stats->>'afk_extra_minutes')::bigint, 0)), 0)::bigint as afk_extra_minutes,
                coalesce(sum(coalesce((it.stats->>'afk_offline_reward_pct')::bigint, 0)), 0)::bigint as afk_offline_reward_pct,
                coalesce(sum(coalesce((it.stats->>'all_skill_bonus')::bigint, 0)), 0)::bigint as all_skill_bonus,
                coalesce(sum(coalesce((it.stats->>'normal_mob_execute_pct')::bigint, 0)), 0)::bigint as normal_mob_execute_pct,
                coalesce(sum(coalesce((it.stats->>'damage_reduce_pct')::bigint, 0)), 0)::bigint as damage_reduce_pct,
                coalesce(sum(coalesce((it.stats->>'paralyze_resist_pct')::bigint, 0)), 0)::bigint as paralyze_resist_pct,
                coalesce(sum(coalesce((it.stats->>'petrify_resist_pct')::bigint, 0)), 0)::bigint as petrify_resist_pct,
                coalesce(sum(coalesce((it.stats->>'ignore_def_pct')::bigint, 0)), 0)::bigint as ignore_def_pct,
                coalesce(sum(coalesce((it.stats->>'guaranteed_hit_pct')::bigint, 0)), 0)::bigint as guaranteed_hit_pct,
                coalesce(sum(coalesce((it.stats->>'target_max_hp_true_damage_pct')::bigint, 0)), 0)::bigint as target_max_hp_true_damage_pct,
                coalesce(sum(coalesce((it.stats->>'self_max_mp_true_damage_pct')::bigint, 0)), 0)::bigint as self_max_mp_true_damage_pct,
                coalesce(sum(coalesce((it.stats->>'creation_strike_pct')::bigint, 0)), 0)::bigint as creation_strike_pct,
                coalesce(sum(coalesce((it.stats->>'creation_strike_damage_pct')::bigint, 0)), 0)::bigint as creation_strike_damage_pct,
                coalesce(bool_or(coalesce((it.stats->>'creation_strike_full_restore')::boolean, false)), false) as creation_strike_full_restore,
                coalesce(bool_or(coalesce((it.stats->>'control_immune')::boolean, false)), false) as control_immune,
                coalesce(bool_or(coalesce((it.stats->>'fatigue_immune')::boolean, false)), false) as fatigue_immune
              from effective_equipped it
            ),
            set_counts as (
              select it.flags->>'set' as set_id, count(*)::bigint as pieces
              from effective_equipped it
              where it.flags ? 'set'
              group by it.flags->>'set'
            ),
            set_bonus as (
              select
                0::bigint as atk_pct,
                coalesce(sum(case when set_id = 'chaos' and pieces >= 3 then 30 else 0 end), 0)::bigint as def_pct,
                coalesce(sum(case when set_id = 'qingyun' and pieces >= 3 then 15 when set_id = 'dominator' and pieces >= 2 then 100 else 0 end), 0)::bigint as hp_pct,
                coalesce(sum(case when set_id = 'qingyun' and pieces >= 3 then 15 when set_id = 'dominator' and pieces >= 2 then 100 else 0 end), 0)::bigint as mp_pct,
                coalesce(sum(case when set_id = 'jiuxiao' and pieces >= 3 then 20 when set_id = 'dominator' and pieces >= 4 then 100 else 0 end), 0)::bigint as skill_damage_pct,
                0::bigint as boss_damage_pct,
                0::bigint as damage_deepen_pct,
                0::bigint as crit_damage_pct,
                coalesce(sum(case when set_id = 'pureyang' and pieces >= 3 then 20 else 0 end), 0)::bigint as battle_end_restore_pct,
                0::bigint as afk_base_reward_pct,
                coalesce(sum(case when set_id = 'qingyun' and pieces >= 5 then 20 when set_id = 'pureyang' and pieces >= 5 then 40 else 0 end), 0)::bigint as afk_exp_pct,
                coalesce(sum(case when set_id = 'qingyun' and pieces >= 5 then 20 else 0 end), 0)::bigint as afk_gold_pct,
                0::bigint as afk_drop_quality_pct,
                0::bigint as afk_kill_pct,
                0::bigint as afk_extra_minutes,
                0::bigint as afk_offline_reward_pct,
                coalesce(sum(case when set_id = 'zaohua' and pieces >= 5 then 10 when set_id = 'dominator' and pieces >= 2 then 20 else 0 end), 0)::bigint as life_steal_pct,
                coalesce(sum(case when set_id = 'zaohua' and pieces >= 5 then 10 when set_id = 'dominator' and pieces >= 2 then 20 else 0 end), 0)::bigint as mana_steal_pct,
                0::bigint as crit_pct,
                coalesce(sum(case when set_id = 'jiuxiao' and pieces >= 6 then 2 else 0 end), 0)::bigint as all_skill_bonus,
                0::bigint as normal_mob_execute_pct,
                coalesce(sum(case when set_id = 'chaos' and pieces >= 5 then 30 else 0 end), 0)::bigint as damage_reduce_pct,
                coalesce(sum(case when set_id = 'zaohua' and pieces >= 3 then 50 when set_id = 'dominator' and pieces >= 4 then 95 else 0 end), 0)::bigint as ignore_def_pct,
                coalesce(sum(case when set_id = 'dominator' and pieces >= 4 then 100 else 0 end), 0)::bigint as guaranteed_hit_pct,
                0::bigint as target_max_hp_true_damage_pct,
                coalesce(sum(case when set_id = 'jiuxiao' and pieces >= 6 then 2 else 0 end), 0)::bigint as self_max_mp_true_damage_pct,
                coalesce(sum(case when set_id = 'dominator' and pieces >= 8 then 15 else 0 end), 0)::bigint as creation_strike_pct,
                coalesce(sum(case when set_id = 'dominator' and pieces >= 8 then 1000 else 0 end), 0)::bigint as creation_strike_damage_pct,
                coalesce(bool_or(set_id = 'dominator' and pieces >= 8), false) as creation_strike_full_restore,
                coalesce(bool_or(set_id = 'dominator' and pieces >= 6), false) as control_immune,
                coalesce(bool_or(set_id = 'dominator' and pieces >= 8), false) as fatigue_immune
              from set_counts
            )
            select
              base_bonus.score,
              base_bonus.atk,
              base_bonus.def,
              base_bonus.mag,
              base_bonus.mdef,
              base_bonus.dex,
              base_bonus.spirit,
              base_bonus.hp,
              base_bonus.mp,
              base_bonus.crit_pct + set_bonus.crit_pct as crit_pct,
              base_bonus.life_steal_pct + set_bonus.life_steal_pct as life_steal_pct,
              base_bonus.mana_steal_pct + set_bonus.mana_steal_pct as mana_steal_pct,
              base_bonus.heavy_hit_pct,
              base_bonus.paralyze_pct,
              base_bonus.petrify_pct,
              base_bonus.atk_pct + set_bonus.atk_pct as atk_pct,
              base_bonus.def_pct + set_bonus.def_pct as def_pct,
              base_bonus.hp_pct + set_bonus.hp_pct as hp_pct,
              base_bonus.mp_pct + set_bonus.mp_pct as mp_pct,
              base_bonus.skill_damage_pct + set_bonus.skill_damage_pct as skill_damage_pct,
              base_bonus.boss_damage_pct + set_bonus.boss_damage_pct as boss_damage_pct,
              base_bonus.damage_deepen_pct + set_bonus.damage_deepen_pct as damage_deepen_pct,
              base_bonus.crit_damage_pct + set_bonus.crit_damage_pct as crit_damage_pct,
              base_bonus.battle_end_restore_pct + set_bonus.battle_end_restore_pct as battle_end_restore_pct,
              base_bonus.afk_base_reward_pct + set_bonus.afk_base_reward_pct as afk_base_reward_pct,
              base_bonus.afk_exp_pct + set_bonus.afk_exp_pct as afk_exp_pct,
              base_bonus.afk_gold_pct + set_bonus.afk_gold_pct as afk_gold_pct,
              base_bonus.afk_drop_quality_pct + set_bonus.afk_drop_quality_pct as afk_drop_quality_pct,
              base_bonus.afk_kill_pct + set_bonus.afk_kill_pct as afk_kill_pct,
              base_bonus.afk_extra_minutes + set_bonus.afk_extra_minutes as afk_extra_minutes,
              base_bonus.afk_offline_reward_pct + set_bonus.afk_offline_reward_pct as afk_offline_reward_pct,
              base_bonus.all_skill_bonus + set_bonus.all_skill_bonus as all_skill_bonus,
              base_bonus.normal_mob_execute_pct + set_bonus.normal_mob_execute_pct as normal_mob_execute_pct,
              base_bonus.damage_reduce_pct + set_bonus.damage_reduce_pct as damage_reduce_pct,
              base_bonus.paralyze_resist_pct as paralyze_resist_pct,
              base_bonus.petrify_resist_pct as petrify_resist_pct,
              base_bonus.ignore_def_pct + set_bonus.ignore_def_pct as ignore_def_pct,
              base_bonus.guaranteed_hit_pct + set_bonus.guaranteed_hit_pct as guaranteed_hit_pct,
              base_bonus.target_max_hp_true_damage_pct + set_bonus.target_max_hp_true_damage_pct as target_max_hp_true_damage_pct,
              base_bonus.self_max_mp_true_damage_pct + set_bonus.self_max_mp_true_damage_pct as self_max_mp_true_damage_pct,
              base_bonus.creation_strike_pct + set_bonus.creation_strike_pct as creation_strike_pct,
              base_bonus.creation_strike_damage_pct + set_bonus.creation_strike_damage_pct as creation_strike_damage_pct,
              base_bonus.creation_strike_full_restore or set_bonus.creation_strike_full_restore as creation_strike_full_restore,
              base_bonus.control_immune or set_bonus.control_immune as control_immune,
              base_bonus.fatigue_immune or set_bonus.fatigue_immune as fatigue_immune
            from base_bonus
            cross join set_bonus
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await
    }

    async fn view_for_character(&self, character_id: i64) -> Result<InventoryView, sqlx::Error> {
        let level: (i32,) = sqlx::query_as("select level from characters where id = $1")
            .bind(character_id)
            .fetch_one(&self.pool)
            .await?;
        self.view(character_id, level.0).await
    }

    async fn inventory_counts(&self, character_id: i64) -> Result<(usize, usize), sqlx::Error> {
        let (bag_used, warehouse_used): (i64, i64) = sqlx::query_as(
            r#"
            select
              count(*) filter (where location = 'bag') as bag_used,
              count(*) filter (where location = 'warehouse') as warehouse_used
            from inventory_items
            where character_id = $1
              and location in ('bag', 'warehouse')
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok((bag_used.max(0) as usize, warehouse_used.max(0) as usize))
    }

    async fn item_rows(
        &self,
        character_id: i64,
        bag_limit: i64,
        warehouse_limit: i64,
    ) -> Result<Vec<InventoryItemRow>, sqlx::Error> {
        sqlx::query_as::<_, InventoryItemRow>(
            r#"
            with selected_items as (
              select *
              from inventory_items
              where character_id = $1 and location = 'equipped'
              union all
              select *
              from (
                select *
                from inventory_items
                where character_id = $1 and location = 'bag'
                order by id asc
                limit $2
              ) bag_items
              union all
              select *
              from (
                select *
                from inventory_items
                where character_id = $1 and location = 'warehouse'
                order by id asc
                limit $3
              ) warehouse_items
            )
            select
              ii.id,
              ii.character_id,
              ii.template_id,
              it.name,
              it.kind,
              it.slot as template_slot,
              it.rarity,
              it.price,
              it.stackable,
              it.stats,
              ii.quantity,
              ii.location,
              ii.slot as equipped_slot,
              ii.bind,
              ii.durability,
              ii.extra
            from selected_items ii
            join item_templates it on it.id = ii.template_id
            order by
              case ii.location when 'equipped' then 0 when 'bag' then 1 else 2 end,
              ii.id asc
            "#,
        )
        .bind(character_id)
        .bind(bag_limit.max(0))
        .bind(warehouse_limit.max(0))
        .fetch_all(&self.pool)
        .await
    }

    async fn character(&self, character_id: i64) -> Result<CharacterRecord, sqlx::Error> {
        sqlx::query_as::<_, CharacterRecord>(
            "select id, account_id, name, class, level, exp, gold, yuanbao, power from characters where id = $1 and deleted_at is null",
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await
    }
}

fn generated_equipment_extra(stats: &serde_json::Value, slot: Option<&str>) -> serde_json::Value {
    if stats.get("set").and_then(serde_json::Value::as_str) == Some("dominator") {
        return serde_json::json!({});
    }
    let tier = stat_i64(stats, "tier");
    if slot.is_none() || tier <= 0 {
        return serde_json::json!({});
    }

    let mut rng = thread_rng();
    let mut generated = serde_json::Map::new();
    for key in ["atk", "mag", "hp", "def", "mdef"] {
        let base = stat_i64(stats, key);
        if base > 0 {
            let pct = rng.gen_range(90..=110);
            generated.insert(key.into(), serde_json::json!((base.saturating_mul(pct) / 100).max(1)));
        }
    }

    if tier >= 6 {
        let cap = ((tier - 5) * 15 / 10).clamp(1, 100);
        let mut pool = vec![
            "damage_deepen_pct",
            "damage_reduce_pct",
            "crit_pct",
            "ignore_def_pct",
            "guaranteed_hit_pct",
            "boss_damage_pct",
        ];
        pool.shuffle(&mut rng);
        let count = rng.gen_range(1..=3).min(pool.len());
        for key in pool.into_iter().take(count) {
            if stat_i64(stats, key) > 0 {
                continue;
            }
            let value = rng.gen_range(1..=cap);
            generated.insert(key.into(), serde_json::json!(value));
        }
    }

    if generated.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::json!({ "generated_stats": generated })
    }
}

fn stat_i64(value: &serde_json::Value, key: &str) -> i64 {
    value.get(key).and_then(|value| value.as_i64()).unwrap_or_default().max(0)
}

fn power_with_bonuses(
    base: &CharacterStats,
    equipment: &EquipmentBonus,
    skills: &SkillBonus,
    systems: &SystemBonus,
) -> i64 {
    let max_hp = apply_pct_i64(
        base.max_hp
            .saturating_add(equipment.hp)
            .saturating_add(skills.hp)
            .saturating_add(systems.hp),
        equipment.hp_pct + systems.hp_pct,
    );
    let max_mp = apply_pct_i64(
        base.max_mp
            .saturating_add(equipment.mp)
            .saturating_add(skills.mp)
            .saturating_add(systems.mp),
        equipment.mp_pct + systems.mp_pct,
    );
    let atk = apply_pct_i64(
        base.atk
            .saturating_add(equipment.atk)
            .saturating_add(skills.atk)
            .saturating_add(systems.atk),
        equipment.atk_pct + systems.atk_pct,
    );
    let def = apply_pct_i64(
        base.def
            .saturating_add(equipment.def)
            .saturating_add(skills.def)
            .saturating_add(systems.def),
        equipment.def_pct + systems.def_pct,
    );
    let mag = apply_pct_i64(
        base.mag
            .saturating_add(equipment.mag)
            .saturating_add(skills.mag)
            .saturating_add(systems.mag),
        equipment.atk_pct + systems.atk_pct,
    );
    let mdef = apply_pct_i64(
        base.mdef
            .saturating_add(equipment.mdef)
            .saturating_add(skills.mdef)
            .saturating_add(systems.mdef),
        equipment.def_pct + systems.def_pct,
    );
    let stats = CharacterStats {
        character_id: base.character_id,
        attrs: Attributes {
            str_: base.attrs.str_,
            dex: base
                .attrs
                .dex
                .saturating_add(equipment.dex)
                .saturating_add(skills.dex)
                .saturating_add(systems.dex),
            int_: base.attrs.int_,
            con: base.attrs.con,
            spirit: base.attrs.spirit.saturating_add(equipment.spirit),
        },
        max_hp,
        max_mp,
        atk,
        def,
        mag,
        mdef,
    };
    power_from_stats(&stats).saturating_add(equipment.score.max(0))
}

fn apply_pct_i64(value: i64, pct: i64) -> i64 {
    value.saturating_mul(100 + pct.clamp(-90, 500)) / 100
}

fn misc_decompose_reward(row: &MiscDecomposeRow) -> (i64, i64, i64) {
    let quantity = row.quantity.max(1);
    let gold = (row.price / 10).max(1) * quantity;
    if row.kind == "book" {
        let pages_each = match row.rarity.as_str() {
            "legendary" => 10,
            "epic" => 5,
            "rare" => 2,
            "uncommon" => 1,
            _ => 1,
        };
        let insight_each = match row.rarity.as_str() {
            "mythic" => 2,
            "legendary" => 1,
            _ => 0,
        };
        return (pages_each * quantity, insight_each * quantity, gold);
    }
    if row.template_id == "skill_page" {
        return (0, quantity / 20, gold);
    }
    (0, 0, gold)
}

#[derive(Debug, Clone, Copy, Default)]
struct EquipmentDecomposeReward {
    refine_stones: i64,
    hongmeng_stones: i64,
    gold: i64,
    yuanbao: i64,
}

impl EquipmentDecomposeReward {
    fn add(&mut self, other: EquipmentDecomposeReward) {
        self.refine_stones = self.refine_stones.saturating_add(other.refine_stones);
        self.hongmeng_stones = self.hongmeng_stones.saturating_add(other.hongmeng_stones);
        self.gold = self.gold.saturating_add(other.gold);
        self.yuanbao = self.yuanbao.saturating_add(other.yuanbao);
    }
}

fn equipment_decompose_reward(
    template_id: &str,
    price: i64,
    quantity: i64,
    enhance_level: i32,
    tier: i64,
    slot: Option<&str>,
    stats: &serde_json::Value,
) -> EquipmentDecomposeReward {
    let quantity = quantity.max(1);
    let mut rng = thread_rng();
    let mut reward = EquipmentDecomposeReward {
        gold: (price / 20).max(1).saturating_mul(quantity),
        yuanbao: yuanbao_decompose_reward(template_id, slot, stats).saturating_mul(quantity),
        ..EquipmentDecomposeReward::default()
    };
    for _ in 0..quantity {
        if rng.gen_range(0..50) == 0 {
            reward.refine_stones += 1;
        }
        if tier >= 9 && rng.gen_range(0..100) == 0 {
            reward.hongmeng_stones += 1;
        }
    }
    if enhance_level > 0 {
        reward.refine_stones = reward.refine_stones.saturating_add(i64::from(enhance_level / 5).max(0));
    }
    reward
}

fn yuanbao_decompose_reward(template_id: &str, slot: Option<&str>, stats: &serde_json::Value) -> i64 {
    let explicit = stat_i64(stats, "yuanbao_decompose");
    if explicit > 0 {
        return explicit;
    }
    let is_dominator = stats
        .get("set")
        .and_then(serde_json::Value::as_str)
        .map(|set| set == "dominator")
        .unwrap_or(false)
        || template_id.starts_with("dominator_");
    if !is_dominator {
        return 0;
    }
    if matches!(slot, Some("weapon")) || template_id.contains("blade") || template_id.contains("weapon") {
        100
    } else {
        10
    }
}

async fn should_auto_decompose_grant(
    pool: &PgPool,
    character_id: i64,
    template: &GrantTemplateRow,
) -> Result<bool, sqlx::Error> {
    let decompose_only = template
        .stats
        .get("decompose_only")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if template.stackable || (template.slot.is_none() && !decompose_only) {
        return Ok(false);
    }
    let tier = stat_i64(&template.stats, "tier");
    if tier <= 0 {
        return Ok(false);
    }
    let row = sqlx::query_as::<_, VipAutoDecomposeRow>(
        r#"
        select
          exists(
            select 1 from vip_records
            where character_id = $1 and (ends_at is null or ends_at > now())
          ) as active_vip,
          coalesce(vps.auto_decompose_enabled, false) as auto_decompose_enabled,
          coalesce(vps.auto_decompose_max_tier, 0) as auto_decompose_max_tier
        from characters c
        left join vip_potion_settings vps on vps.character_id = c.id
        where c.id = $1 and c.deleted_at is null
        "#,
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(false);
    };
    Ok(row.active_vip
        && row.auto_decompose_enabled
        && row.auto_decompose_max_tier > 0
        && tier <= i64::from(row.auto_decompose_max_tier))
}

async fn should_auto_extract_essence_grant(
    pool: &PgPool,
    character_id: i64,
    template_id: &str,
    quantity: i64,
    template: &GrantTemplateRow,
) -> Result<bool, sqlx::Error> {
    if template.stackable || template.slot.is_none() {
        return Ok(false);
    }
    let tier = stat_i64(&template.stats, "tier");
    if tier <= 0 {
        return Ok(false);
    }
    let row = sqlx::query_as::<_, VipAutoExtractRow>(
        r#"
        select
          exists(
            select 1 from vip_records
            where character_id = $1 and (ends_at is null or ends_at > now())
          ) as active_vip,
          exists(
            select 1 from character_system_unlocks
            where character_id = $1 and system_id = 'wanxiang'
          ) as wanxiang_unlocked,
          coalesce(vps.auto_extract_essence_enabled, false) as auto_extract_essence_enabled,
          coalesce(vps.auto_extract_essence_max_tier, 0) as auto_extract_essence_max_tier
        from characters c
        left join vip_potion_settings vps on vps.character_id = c.id
        where c.id = $1 and c.deleted_at is null
        "#,
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(false);
    };
    if !(row.active_vip
        && row.wanxiang_unlocked
        && row.auto_extract_essence_enabled
        && row.auto_extract_essence_max_tier > 0
        && tier <= i64::from(row.auto_extract_essence_max_tier))
    {
        return Ok(false);
    }
    let essence = equipment_essence_value(template_id, &template.stats).saturating_mul(quantity.max(1));
    if essence <= 0 {
        return Ok(false);
    }
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
    .bind(essence)
    .execute(pool)
    .await?;
    Ok(true)
}

fn equipment_essence_value(template_id: &str, stats: &serde_json::Value) -> i64 {
    if stats.get("set").and_then(serde_json::Value::as_str) == Some("dominator")
        || template_id.starts_with("dominator_")
    {
        return 100;
    }
    stat_i64(stats, "tier").clamp(1, 17)
}

async fn grant_equipment_decompose_rewards(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    reward: &EquipmentDecomposeReward,
) -> Result<(), sqlx::Error> {
    if reward.gold > 0 || reward.yuanbao > 0 {
        sqlx::query(
            r#"
            update characters
            set
              gold = least(9223372036854775807::numeric, greatest(0::numeric, gold::numeric + $2::numeric))::bigint,
              yuanbao = least(9223372036854775807::numeric, greatest(0::numeric, yuanbao::numeric + $3::numeric))::bigint
            where id = $1
            "#,
        )
            .bind(character_id)
            .bind(reward.gold)
            .bind(reward.yuanbao)
            .execute(&mut **tx)
            .await?;
    }
    grant_decompose_stackable(tx, character_id, "stone_refine", reward.refine_stones).await?;
    grant_decompose_stackable(tx, character_id, "stone_hongmeng", reward.hongmeng_stones).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct EnhanceMaterial {
    template_id: &'static str,
    name: &'static str,
    quantity: i64,
}

#[derive(Debug, Clone, Copy)]
enum EnhanceFailurePenalty {
    Keep,
    Downgrade,
    Destroy,
}

async fn material_quantity(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
) -> Result<i64, sqlx::Error> {
    let (quantity,): (i64,) = sqlx::query_as(
        r#"
        select coalesce(sum(quantity), 0)::bigint
        from inventory_items
        where character_id = $1
          and template_id = $2
          and location = 'bag'
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(quantity)
}

async fn consume_material(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, StackRow>(
        r#"
        select id, quantity
        from inventory_items
        where character_id = $1
          and template_id = $2
          and location = 'bag'
        order by id asc
        for update
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .fetch_all(&mut **tx)
    .await?;

    let mut remaining = quantity.max(0);
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

async fn consume_stackable(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), InventoryActionError> {
    let quantity = quantity.max(1);
    let available = material_quantity(tx, character_id, template_id).await?;
    if available < quantity {
        return Err(InventoryActionError::NotEnoughMaterial(template_id.to_string()));
    }
    consume_material(tx, character_id, template_id, quantity).await?;
    Ok(())
}

async fn bag_has_room_for_new_rows(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    rows_needed: i64,
) -> Result<bool, sqlx::Error> {
    if rows_needed <= 0 {
        return Ok(true);
    }
    let (level,): (i32,) =
        sqlx::query_as("select level from characters where id = $1 and deleted_at is null for update")
            .bind(character_id)
            .fetch_one(&mut **tx)
            .await?;
    let (bag_count,): (i64,) =
        sqlx::query_as("select count(*)::bigint from inventory_items where character_id = $1 and location = 'bag'")
            .bind(character_id)
            .fetch_one(&mut **tx)
            .await?;
    Ok(bag_count.saturating_add(rows_needed) <= (bag_limit_for_level(level) as i64))
}

async fn grant_stackable(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), sqlx::Error> {
    if quantity <= 0 {
        return Ok(());
    }
    let updated = sqlx::query(
        r#"
        update inventory_items
        set quantity = quantity + $3
        where id = (
          select id
          from inventory_items
          where character_id = $1 and template_id = $2 and location = 'bag'
          order by id asc
          limit 1
        )
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .bind(quantity)
    .execute(&mut **tx)
    .await?;
    if updated.rows_affected() == 0 {
        if !bag_has_room_for_new_rows(tx, character_id, 1).await? {
            let warehouse_updated = sqlx::query(
                r#"
                update inventory_items
                set quantity = quantity + $3
                where id = (
                  select id
                  from inventory_items
                  where character_id = $1 and template_id = $2 and location = 'warehouse'
                  order by id asc
                  limit 1
                )
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity)
            .execute(&mut **tx)
            .await?;
            if warehouse_updated.rows_affected() == 0 {
                sqlx::query(
                    "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, $3, 'warehouse')",
                )
                .bind(character_id)
                .bind(template_id)
                .bind(quantity)
                .execute(&mut **tx)
                .await?;
            }
            return Ok(());
        }
        sqlx::query(
            "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, $3, 'bag')",
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn grant_decompose_stackable(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), sqlx::Error> {
    if quantity <= 0 {
        return Ok(());
    }
    let updated = sqlx::query(
        r#"
        update inventory_items
        set quantity = quantity + $3
        where id = (
          select id
          from inventory_items
          where character_id = $1
            and template_id = $2
            and location in ('bag', 'warehouse')
          order by case location when 'bag' then 0 else 1 end, id asc
          limit 1
        )
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .bind(quantity)
    .execute(&mut **tx)
    .await?;
    if updated.rows_affected() == 0 {
        sqlx::query(
            "insert into inventory_items (character_id, template_id, quantity, location) values ($1, $2, $3, 'warehouse')",
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

fn enhance_material_requirements(next_level: i32) -> Vec<EnhanceMaterial> {
    let mut materials = vec![EnhanceMaterial {
        template_id: "stone_refine",
        name: "炼器石",
        quantity: i64::from(next_level.max(1)),
    }];
    if next_level > 10 {
        materials.push(EnhanceMaterial {
            template_id: "stone_hongmeng",
            name: "鸿蒙石",
            quantity: i64::from((next_level - 10).max(1)),
        });
    }
    materials
}

fn enhance_failure_penalty(next_level: i32) -> EnhanceFailurePenalty {
    match next_level {
        1..=10 => EnhanceFailurePenalty::Keep,
        11..=15 => EnhanceFailurePenalty::Downgrade,
        _ => EnhanceFailurePenalty::Destroy,
    }
}

fn format_materials(materials: &[EnhanceMaterial]) -> String {
    materials
        .iter()
        .map(|material| format!("{} x{}", material.name, material.quantity))
        .collect::<Vec<_>>()
        .join("、")
}

fn enhance_success_rate(next_level: i32) -> f64 {
    match next_level {
        1..=3 => 0.96,
        4..=6 => 0.86,
        7..=10 => 0.72,
        11..=13 => 0.58,
        14..=15 => 0.48,
        16..=18 => 0.34,
        _ => 0.24,
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

fn item_view(row: InventoryItemRow) -> PlayerInventoryItemView {
    let mut stats = row.stats;
    let mut display_name = row.name;
    if let (Some(stats_object), Some(generated_object)) = (
        stats.as_object_mut(),
        row.extra.get("generated_stats").and_then(serde_json::Value::as_object),
    ) {
        for (key, value) in generated_object {
            stats_object.insert(key.clone(), value.clone());
        }
    }
    if let (Some(stats_object), Some(extra_object)) = (stats.as_object_mut(), row.extra.as_object()) {
        if let Some(enhance) = extra_object.get("enhance").and_then(serde_json::Value::as_i64) {
            if enhance > 0 {
                let bonus = enhance_bonus(&row.kind, row.template_slot.as_deref(), enhance);
                for (key, value) in bonus {
                    let current = stats_object.get(key).and_then(serde_json::Value::as_i64).unwrap_or_default();
                    stats_object.insert(key.into(), serde_json::json!(current + value));
                }
                stats_object.insert("enhance".into(), serde_json::json!(enhance));
                display_name = format!("{} +{}", display_name, enhance);
            }
        }
    }
    PlayerInventoryItemView {
        id: row.id,
        character_id: row.character_id,
        template_id: row.template_id,
        name: display_name,
        kind: row.kind,
        template_slot: row.template_slot,
        rarity: row.rarity,
        price: row.price,
        stackable: row.stackable,
        stats,
        quantity: row.quantity,
        location: row.location,
        equipped_slot: row.equipped_slot,
        bind: row.bind,
        durability: row.durability,
    }
}

fn enhance_bonus(kind: &str, slot: Option<&str>, enhance: i64) -> Vec<(&'static str, i64)> {
    match (kind, slot.unwrap_or_default()) {
        ("weapon", _) => {
            let mut bonus = vec![("atk", enhance * 4)];
            if enhance >= 11 {
                bonus.push(("life_steal_pct", enhance - 10));
            }
            bonus
        }
        ("armor", "chest") => vec![("def", enhance * 2), ("mdef", enhance)],
        ("armor", _) => vec![("def", enhance * 2)],
        ("accessory", _) => vec![("atk", enhance), ("mag", enhance), ("dex", enhance)],
        _ => vec![("atk", enhance), ("def", enhance)],
    }
}

fn summarize_inventory(
    rows: &[InventoryItemRow],
    level: i32,
    bag_used: usize,
    warehouse_used: usize,
) -> PlayerInventorySummary {
    let mut equipment = PlayerEquipmentSlots::default();

    for row in rows {
        if row.location == "equipped" {
            set_equipment_slot(&mut equipment, row.equipped_slot.as_deref(), row.id);
        }
    }

    PlayerInventorySummary {
        bag_used,
        bag_limit: bag_limit_for_level(level),
        warehouse_used,
        equipment,
    }
}

fn set_equipment_slot(equipment: &mut PlayerEquipmentSlots, slot: Option<&str>, item_id: i64) {
    match slot {
        Some("weapon") => equipment.weapon = Some(item_id),
        Some("chest") => equipment.chest = Some(item_id),
        Some("head") => equipment.head = Some(item_id),
        Some("feet") => equipment.feet = Some(item_id),
        Some("waist") => equipment.waist = Some(item_id),
        Some("neck") => equipment.neck = Some(item_id),
        Some("ring_left") => equipment.ring_left = Some(item_id),
        Some("ring_right") => equipment.ring_right = Some(item_id),
        Some("bracelet_left") => equipment.bracelet_left = Some(item_id),
        Some("bracelet_right") => equipment.bracelet_right = Some(item_id),
        _ => {}
    }
}

async fn resolve_equip_slot(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    item_id: i64,
    template_slot: &str,
) -> Result<String, sqlx::Error> {
    let Some(candidates) = paired_equipment_slots(template_slot) else {
        return Ok(template_slot.to_owned());
    };
    let occupied = sqlx::query_as::<_, (String,)>(
        r#"
        select slot
        from inventory_items
        where character_id = $1
          and location = 'equipped'
          and slot in ($2, $3)
          and id <> $4
        "#,
    )
    .bind(character_id)
    .bind(candidates[0])
    .bind(candidates[1])
    .bind(item_id)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| row.0)
    .collect::<Vec<_>>();

    for slot in candidates {
        if !occupied.iter().any(|occupied_slot| occupied_slot == slot) {
            return Ok(slot.to_owned());
        }
    }
    Ok(candidates[0].to_owned())
}

fn paired_equipment_slots(slot: &str) -> Option<[&'static str; 2]> {
    match slot {
        "ring_left" => Some(["ring_left", "ring_right"]),
        "ring_right" => Some(["ring_right", "ring_left"]),
        "bracelet_left" => Some(["bracelet_left", "bracelet_right"]),
        "bracelet_right" => Some(["bracelet_right", "bracelet_left"]),
        _ => None,
    }
}
