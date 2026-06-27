use cq_protocol::dto::PlayerSkillView;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;

const SKILL_MAX_LEVEL: i32 = 100;
const SKILL_TOTAL_PROFICIENCY: i64 = 5_184_000;

#[derive(Debug, Clone, FromRow)]
struct SkillRow {
    id: String,
    name: String,
    class: String,
    min_level: i32,
    mp_cost: i64,
    cooldown_ms: i32,
    config: serde_json::Value,
    learned: bool,
    auto_enabled: bool,
    level: Option<i32>,
    proficiency: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ActiveSkillRecord {
    pub id: String,
    pub name: String,
    pub mp_cost: i64,
    pub cooldown_ms: i32,
    pub config: serde_json::Value,
    pub level: i32,
}

#[derive(Debug, Clone, FromRow)]
struct CharacterClassLevel {
    class: String,
    level: i32,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct SkillBonus {
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
    pub dex: i64,
    pub hp: i64,
    pub mp: i64,
    pub crit_pct: i64,
    pub crit_damage_pct: i64,
    pub control_resist_pct: i64,
    pub atk_pct: i64,
    pub mag_pct: i64,
    pub def_pct: i64,
    pub hp_pct: i64,
    pub mp_pct: i64,
    pub life_steal_pct: i64,
    pub mana_steal_pct: i64,
    pub damage_reduce_pct: i64,
    pub ignore_def_pct: i64,
    pub guaranteed_hit_pct: i64,
    pub damage_deepen_pct: i64,
}

#[derive(Debug, Clone)]
pub struct SkillUpgradeOutcome {
    pub skill: PlayerSkillView,
    pub succeeded: bool,
    pub consumed_pills: i64,
}

#[derive(Debug, Error)]
pub enum SkillUpgradeError {
    #[error("skill not found")]
    NotFound,
    #[error("skill is max level")]
    MaxLevel,
    #[error("not enough proficiency")]
    NotEnoughProficiency { required: i64, current: i64 },
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct SkillRepository {
    pool: PgPool,
}

impl SkillRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_for_character(&self, character_id: i64) -> Result<Vec<PlayerSkillView>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SkillRow>(
            r#"
            select
              s.id,
              s.name,
              s.class,
              s.min_level,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              cs.skill_id is not null as learned,
              coalesce(cs.auto_enabled, true) as auto_enabled,
              cs.level,
              cs.proficiency
            from skills s
            join characters c on c.id = $1 and c.deleted_at is null
            left join character_skills cs on cs.character_id = c.id and cs.skill_id = s.id
            where s.class in ('all', c.class)
            order by s.min_level asc, s.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(skill_view).collect())
    }

    pub async fn learn(&self, character_id: i64, skill_id: &str) -> Result<PlayerSkillView, sqlx::Error> {
        let character = sqlx::query_as::<_, CharacterClassLevel>(
            r#"
            select class, level
            from characters
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;

        let skill = sqlx::query_as::<_, SkillRow>(
            r#"
            select
              s.id,
              s.name,
              s.class,
              s.min_level,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              false as learned,
              true as auto_enabled,
              null::integer as level,
              null::bigint as proficiency
            from skills s
            where s.id = $1
            "#,
        )
        .bind(skill_id)
        .fetch_one(&self.pool)
        .await?;

        if skill.class != "all" && skill.class != character.class {
            return Err(sqlx::Error::RowNotFound);
        }
        if character.level < skill.min_level {
            return Err(sqlx::Error::RowNotFound);
        }
        if skill_special_upgrade_only(&skill.config) {
            return Err(sqlx::Error::RowNotFound);
        }

        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query(
            r#"
            insert into character_skills (character_id, skill_id, level, proficiency)
            values ($1, $2, 1, 0)
            on conflict (character_id, skill_id) do nothing
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .execute(&mut *tx)
        .await?;
        if inserted.rows_affected() > 0 && skill_requires_book(&skill.config) {
            let book_id = skill_book_id(&skill.config).ok_or(sqlx::Error::RowNotFound)?;
            if !consume_stackable(&mut tx, character_id, book_id, 1).await? {
                return Err(sqlx::Error::RowNotFound);
            }
        }
        if inserted.rows_affected() > 0 {
            let gold_cost = skill_gold_cost(&skill.config);
            if gold_cost > 0 && !debit_gold(&mut tx, character_id, gold_cost).await? {
                return Err(sqlx::Error::RowNotFound);
            }
        }
        tx.commit().await?;

        self.get_for_character(character_id, skill_id).await
    }

    pub async fn upgrade(
        &self,
        character_id: i64,
        skill_id: &str,
    ) -> Result<SkillUpgradeOutcome, SkillUpgradeError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, SkillRow>(
            r#"
            select
              s.id,
              s.name,
              s.class,
              s.min_level,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              true as learned,
              coalesce(cs.auto_enabled, true) as auto_enabled,
              cs.level,
              cs.proficiency
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.class in ('all', c.class)
            for update of cs
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(SkillUpgradeError::NotFound)?;
        if skill_special_upgrade_only(&row.config) {
            return Err(SkillUpgradeError::NotFound);
        }

        let level = row.level.unwrap_or(1);
        let proficiency = row.proficiency.unwrap_or_default();
        if level >= SKILL_MAX_LEVEL {
            return Err(SkillUpgradeError::MaxLevel);
        }
        let target_level = skill_level_for_proficiency(proficiency)
            .max(level)
            .clamp(1, SKILL_MAX_LEVEL);
        if target_level <= level {
            let required_proficiency = skill_proficiency_for_level(level.saturating_add(1));
            return Err(SkillUpgradeError::NotEnoughProficiency { required: required_proficiency, current: proficiency });
        }

        sqlx::query(
            r#"
            update character_skills
            set level = $3,
                updated_at = now()
            where character_id = $1 and skill_id = $2
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .bind(target_level)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(SkillUpgradeOutcome {
            skill: self.get_for_character(character_id, skill_id).await?,
            succeeded: true,
            consumed_pills: 0,
        })
    }

    pub async fn bonus(&self, character_id: i64) -> Result<SkillBonus, sqlx::Error> {
        sqlx::query_as::<_, SkillBonus>(
            r#"
            select
              coalesce(sum(coalesce((s.config->>'atk_bonus')::bigint, 0) * cs.level), 0)::bigint as atk,
              coalesce(sum(coalesce((s.config->>'def_bonus')::bigint, 0) * cs.level), 0)::bigint as def,
              coalesce(sum(coalesce((s.config->>'mag_bonus')::bigint, 0) * cs.level), 0)::bigint as mag,
              coalesce(sum(coalesce((s.config->>'mdef_bonus')::bigint, 0) * cs.level), 0)::bigint as mdef,
              coalesce(sum((coalesce((s.config->>'dex_bonus')::bigint, 0) + coalesce((s.config->>'luck_bonus')::bigint, 0)) * cs.level), 0)::bigint
                + coalesce(sum(coalesce((s.config->>'luck_bonus_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as dex,
              coalesce(sum(coalesce((s.config->>'hp_bonus')::bigint, 0) * cs.level), 0)::bigint as hp,
              coalesce(sum(coalesce((s.config->>'mp_bonus')::bigint, 0) * cs.level), 0)::bigint as mp,
              coalesce(sum((coalesce((s.config->>'crit_bonus')::bigint, 0) + coalesce((s.config->>'crit_pct_bonus')::bigint, 0)) * cs.level), 0)::bigint as crit_pct,
              coalesce(sum(coalesce((s.config->>'crit_damage_pct_bonus')::bigint, 0) * cs.level), 0)::bigint as crit_damage_pct,
              coalesce(sum(coalesce((s.config->>'control_resist_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as control_resist_pct,
              coalesce(sum(coalesce((s.config->>'atk_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as atk_pct,
              coalesce(sum(coalesce((s.config->>'mag_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as mag_pct,
              coalesce(sum(coalesce((s.config->>'def_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as def_pct,
              coalesce(sum(coalesce((s.config->>'hp_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as hp_pct,
              coalesce(sum(coalesce((s.config->>'mp_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as mp_pct,
              coalesce(sum(
                coalesce((s.config->>'life_steal_base_pct')::bigint, 0)
                + coalesce((s.config->>'life_steal_pct_per_100')::bigint, 0) * cs.level / 100
              ), 0)::bigint as life_steal_pct,
              coalesce(sum(
                coalesce((s.config->>'mana_steal_base_pct')::bigint, 0)
                + coalesce((s.config->>'mana_steal_pct_per_100')::bigint, 0) * cs.level / 100
              ), 0)::bigint as mana_steal_pct,
              coalesce(sum(coalesce((s.config->>'damage_reduce_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as damage_reduce_pct,
              coalesce(sum(coalesce((s.config->>'ignore_def_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as ignore_def_pct,
              coalesce(sum(coalesce((s.config->>'guaranteed_hit_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as guaranteed_hit_pct,
              coalesce(sum(coalesce((s.config->>'damage_deepen_pct_per_100')::bigint, 0) * cs.level / 100), 0)::bigint as damage_deepen_pct
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and s.class in ('all', c.class)
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn active_for_character(
        &self,
        character_id: i64,
        skill_id: &str,
    ) -> Result<ActiveSkillRecord, sqlx::Error> {
        sqlx::query_as::<_, ActiveSkillRecord>(
            r#"
            select
              s.id,
              s.name,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              cs.level
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.class in ('all', c.class)
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn learned_for_character(
        &self,
        character_id: i64,
        skill_id: &str,
    ) -> Result<PlayerSkillView, sqlx::Error> {
        let row = sqlx::query_as::<_, SkillRow>(
            r#"
            select
              s.id,
              s.name,
              s.class,
              s.min_level,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              true as learned,
              coalesce(cs.auto_enabled, true) as auto_enabled,
              cs.level,
              cs.proficiency
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.class in ('all', c.class)
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(skill_view(row))
    }

    pub async fn set_auto_enabled(
        &self,
        character_id: i64,
        skill_id: &str,
        auto_enabled: bool,
    ) -> Result<PlayerSkillView, sqlx::Error> {
        let result = sqlx::query(
            r#"
            update character_skills cs
            set auto_enabled = $3,
                updated_at = now()
            from skills s, characters c
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.id = cs.skill_id
              and c.id = cs.character_id
              and c.deleted_at is null
              and s.class in ('all', c.class)
              and coalesce(s.config->>'kind', 'physical') <> 'passive'
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .bind(auto_enabled)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        self.get_for_character(character_id, skill_id).await
    }

    pub async fn active_skills_for_character(
        &self,
        character_id: i64,
    ) -> Result<Vec<ActiveSkillRecord>, sqlx::Error> {
        sqlx::query_as::<_, ActiveSkillRecord>(
            r#"
            select
              s.id,
              s.name,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              cs.level
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and s.class in ('all', c.class)
              and coalesce(s.config->>'kind', 'physical') <> 'passive'
              and coalesce(cs.auto_enabled, true) = true
            order by
              coalesce(nullif(s.config->>'auto_priority', '')::integer, s.min_level) desc,
              s.min_level desc,
              cs.level desc,
              s.mp_cost desc,
              s.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_used(
        &self,
        character_id: i64,
        skill_id: &str,
        cooldown_ms: i32,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            update character_skills cs
            set last_used_at = now()
            from skills s, characters c
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.id = cs.skill_id
              and c.id = cs.character_id
              and c.deleted_at is null
              and s.class in ('all', c.class)
              and coalesce(s.config->>'kind', 'physical') <> 'passive'
              and (
                cs.last_used_at is null
                or cs.last_used_at <= now() - (($3::text || ' milliseconds')::interval)
              )
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .bind(cooldown_ms.max(0))
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        let mut tx = self.pool.begin().await?;
        add_skill_proficiency_tx(&mut tx, character_id, skill_id, 1).await?;
        let passive_ids = sqlx::query_as::<_, (String,)>(
            r#"
            select cs.skill_id
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and s.class in ('all', c.class)
              and coalesce(s.config->>'kind', 'physical') = 'passive'
              and coalesce((s.config->>'special_upgrade_only')::boolean, false) = false
            "#,
        )
        .bind(character_id)
        .fetch_all(&mut *tx)
        .await?;
        for (passive_id,) in passive_ids {
            add_skill_proficiency_tx(&mut tx, character_id, &passive_id, 1).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn reset_active_cooldowns(&self, character_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            update character_skills cs
            set last_used_at = null
            from skills s, characters c
            where cs.character_id = $1
              and s.id = cs.skill_id
              and c.id = cs.character_id
              and c.deleted_at is null
              and s.class in ('all', c.class)
              and coalesce(s.config->>'kind', 'physical') <> 'passive'
            "#,
        )
        .bind(character_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_study_proficiency(
        &self,
        character_id: i64,
        skill_id: &str,
        amount: i64,
    ) -> Result<(), sqlx::Error> {
        if amount <= 0 {
            return Ok(());
        }
        let exists: Option<(String,)> = sqlx::query_as(
            r#"
            select cs.skill_id
            from character_skills cs
            join skills s on s.id = cs.skill_id
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.character_id = $1
              and cs.skill_id = $2
              and s.class in ('all', c.class)
              and coalesce((s.config->>'special_upgrade_only')::boolean, false) = false
            "#,
        )
        .bind(character_id)
        .bind(skill_id.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some((skill_id,)) = exists else {
            return Err(sqlx::Error::RowNotFound);
        };
        let mut tx = self.pool.begin().await?;
        add_skill_proficiency_tx(&mut tx, character_id, &skill_id, amount).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_for_character(&self, character_id: i64, skill_id: &str) -> Result<PlayerSkillView, sqlx::Error> {
        let row = sqlx::query_as::<_, SkillRow>(
            r#"
            select
              s.id,
              s.name,
              s.class,
              s.min_level,
              s.mp_cost,
              s.cooldown_ms,
              s.config,
              cs.skill_id is not null as learned,
              coalesce(cs.auto_enabled, true) as auto_enabled,
              cs.level,
              cs.proficiency
            from skills s
            join characters c on c.id = $1 and c.deleted_at is null
            left join character_skills cs on cs.character_id = c.id and cs.skill_id = s.id
            where s.id = $2 and s.class in ('all', c.class)
            "#,
        )
        .bind(character_id)
        .bind(skill_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(skill_view(row))
    }
}

fn skill_view(row: SkillRow) -> PlayerSkillView {
    PlayerSkillView {
        id: row.id,
        name: row.name,
        class: row.class,
        min_level: row.min_level,
        mp_cost: row.mp_cost,
        cooldown_ms: row.cooldown_ms,
        config: row.config,
        learned: row.learned,
        auto_enabled: row.auto_enabled,
        level: row.level,
        proficiency: row.proficiency,
    }
}

#[derive(Debug, FromRow)]
struct StackRow {
    id: i64,
    quantity: i64,
}

async fn stackable_quantity(
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

async fn consume_stackable(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<bool, sqlx::Error> {
    let available = stackable_quantity(tx, character_id, template_id).await?;
    if available < quantity {
        return Ok(false);
    }
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
    Ok(true)
}

async fn debit_gold(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    amount: i64,
) -> Result<bool, sqlx::Error> {
    if amount <= 0 {
        return Ok(true);
    }
    let updated = sqlx::query(
        r#"
        update characters
        set gold = gold - $2
        where id = $1 and gold >= $2 and deleted_at is null
        "#,
    )
    .bind(character_id)
    .bind(amount)
    .execute(&mut **tx)
    .await?;
    Ok(updated.rows_affected() > 0)
}

async fn add_skill_proficiency_tx(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    skill_id: &str,
    amount: i64,
) -> Result<(), sqlx::Error> {
    if amount <= 0 {
        return Ok(());
    }
    let Some((current_proficiency, current_level)): Option<(i64, i32)> = sqlx::query_as(
        r#"
        select proficiency, level
        from character_skills
        where character_id = $1 and skill_id = $2
        for update
        "#,
    )
    .bind(character_id)
    .bind(skill_id)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Err(sqlx::Error::RowNotFound);
    };
    let next_proficiency = current_proficiency
        .saturating_add(amount)
        .clamp(0, SKILL_TOTAL_PROFICIENCY);
    let next_level = skill_level_for_proficiency(next_proficiency).max(current_level).clamp(1, SKILL_MAX_LEVEL);
    sqlx::query(
        r#"
        update character_skills
        set proficiency = $3,
            level = $4,
            updated_at = now()
        where character_id = $1 and skill_id = $2
        "#,
    )
    .bind(character_id)
    .bind(skill_id)
    .bind(next_proficiency)
    .bind(next_level)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn skill_requires_book(config: &serde_json::Value) -> bool {
    config.get("requires_book").and_then(serde_json::Value::as_bool).unwrap_or(true)
}

fn skill_special_upgrade_only(config: &serde_json::Value) -> bool {
    config
        .get("special_upgrade_only")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn skill_book_id(config: &serde_json::Value) -> Option<&str> {
    config.get("book_id").and_then(serde_json::Value::as_str)
}

fn skill_gold_cost(config: &serde_json::Value) -> i64 {
    config.get("gold_cost").and_then(serde_json::Value::as_i64).unwrap_or_default().max(0)
}

pub fn skill_level_for_proficiency(proficiency: i64) -> i32 {
    let proficiency = proficiency.clamp(0, SKILL_TOTAL_PROFICIENCY);
    let mut level = 1;
    for candidate in 2..=SKILL_MAX_LEVEL {
        if proficiency >= skill_proficiency_for_level(candidate) {
            level = candidate;
        } else {
            break;
        }
    }
    level
}

pub fn skill_proficiency_for_level(level: i32) -> i64 {
    let level = level.clamp(1, SKILL_MAX_LEVEL);
    if level <= 1 {
        return 0;
    }
    if level >= SKILL_MAX_LEVEL {
        return SKILL_TOTAL_PROFICIENCY;
    }
    let step = i64::from(level - 1);
    let max_step = i64::from(SKILL_MAX_LEVEL - 1);
    SKILL_TOTAL_PROFICIENCY
        .saturating_mul(step)
        .saturating_mul(step)
        / max_step.saturating_mul(max_step).max(1)
}
