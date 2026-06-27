use cq_protocol::dto::PlayerQuestView;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;

#[derive(Debug, Clone, FromRow)]
struct QuestViewRow {
    id: String,
    category: String,
    name: String,
    description: String,
    objectives: Value,
    rewards: Value,
    progress: i64,
    required: i64,
    completed_at: Option<String>,
    claimed_at: Option<String>,
    sort_order: i32,
}

#[derive(Debug, Clone, FromRow)]
struct ProgressQuestRow {
    id: String,
    required: i64,
    period_key: String,
}

#[derive(Debug, Clone, FromRow)]
struct QuestTemplateRow {
    id: String,
    name: String,
    objectives: Value,
    rewards: Value,
    required: i64,
    period_key: String,
}

#[derive(Debug, Clone, FromRow)]
struct QuestClaimRow {
    progress: i64,
    completed_at: Option<String>,
    claimed_at: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct ItemTemplateRow {
    name: String,
    stackable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct QuestRewardSummary {
    pub quest_name: String,
    pub gold: i64,
    pub items: Vec<String>,
}

#[derive(Debug, Error)]
pub enum QuestError {
    #[error("quest not found")]
    NotFound,
    #[error("quest is not complete")]
    NotComplete,
    #[error("quest reward already claimed")]
    AlreadyClaimed,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct QuestRepository {
    pool: PgPool,
}

impl QuestRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_for_character(
        &self,
        character_id: i64,
    ) -> Result<Vec<PlayerQuestView>, sqlx::Error> {
        self.sync_static_objectives(character_id).await?;
        let rows = sqlx::query_as::<_, QuestViewRow>(
            r#"
            select
              qt.id,
              qt.category,
              qt.name,
              qt.description,
              qt.objectives,
              qt.rewards,
              coalesce((cq.progress->>'value')::bigint, 0)::bigint as progress,
              greatest(coalesce((qt.objectives->>'required')::bigint, 1), 1)::bigint as required,
              cq.completed_at::text as completed_at,
              cq.claimed_at::text as claimed_at,
              qt.sort_order
            from quest_templates qt
            join characters c on c.id = $1 and c.deleted_at is null and c.level >= qt.min_level
            left join character_quests cq
              on cq.character_id = c.id
             and cq.quest_id = qt.id
             and cq.period_key = case when qt.category = 'daily' then current_date::text else 'once' end
            where qt.enabled = true
            order by qt.sort_order asc, qt.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(quest_view).collect())
    }

    pub async fn add_progress(
        &self,
        character_id: i64,
        objective_kind: &str,
        amount: i64,
    ) -> Result<(), sqlx::Error> {
        if amount <= 0 || objective_kind.trim().is_empty() {
            return Ok(());
        }
        let rows = progress_quests(&self.pool, objective_kind).await?;
        let mut tx = self.pool.begin().await?;
        for row in rows {
            ensure_progress_row(&mut tx, character_id, &row).await?;
            increment_progress_row(&mut tx, character_id, &row, amount).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn claim(
        &self,
        character_id: i64,
        quest_id: &str,
    ) -> Result<QuestRewardSummary, QuestError> {
        self.sync_static_objectives(character_id).await?;
        let template = sqlx::query_as::<_, QuestTemplateRow>(
            r#"
            select
              qt.id,
              qt.name,
              qt.objectives,
              qt.rewards,
              greatest(coalesce((qt.objectives->>'required')::bigint, 1), 1)::bigint as required,
              case when qt.category = 'daily' then current_date::text else 'once' end as period_key
            from quest_templates qt
            join characters c on c.id = $2 and c.deleted_at is null and c.level >= qt.min_level
            where qt.id = $1 and qt.enabled = true
            "#,
        )
        .bind(quest_id)
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(QuestError::NotFound)?;

        let row = ProgressQuestRow {
            id: template.id.clone(),
            required: template.required,
            period_key: template.period_key.clone(),
        };
        let mut tx = self.pool.begin().await?;
        ensure_progress_row(&mut tx, character_id, &row).await?;
        sync_claimable_progress(&mut tx, character_id, &template).await?;

        let current = sqlx::query_as::<_, QuestClaimRow>(
            r#"
            select
              coalesce((progress->>'value')::bigint, 0)::bigint as progress,
              completed_at::text as completed_at,
              claimed_at::text as claimed_at
            from character_quests
            where character_id = $1 and quest_id = $2 and period_key = $3
            for update
            "#,
        )
        .bind(character_id)
        .bind(&template.id)
        .bind(&template.period_key)
        .fetch_one(&mut *tx)
        .await?;

        if current.claimed_at.is_some() {
            return Err(QuestError::AlreadyClaimed);
        }
        if current.completed_at.is_none() && current.progress < template.required {
            return Err(QuestError::NotComplete);
        }

        let mut summary = QuestRewardSummary {
            quest_name: template.name.clone(),
            ..Default::default()
        };
        apply_rewards(&mut tx, character_id, &template.rewards, &mut summary).await?;

        sqlx::query(
            r#"
            update character_quests
            set completed_at = coalesce(completed_at, now()),
                claimed_at = now()
            where character_id = $1 and quest_id = $2 and period_key = $3 and claimed_at is null
            "#,
        )
        .bind(character_id)
        .bind(&template.id)
        .bind(&template.period_key)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(summary)
    }

    async fn sync_static_objectives(&self, character_id: i64) -> Result<(), sqlx::Error> {
        let Some((level, has_guild)) = sqlx::query_as::<_, (i32, bool)>(
            r#"
            select
              c.level,
              exists(select 1 from guild_members gm where gm.character_id = c.id) as has_guild
            from characters c
            where c.id = $1 and c.deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(());
        };

        self.set_progress_at_least(character_id, "level", i64::from(level)).await?;
        if has_guild {
            self.set_progress_at_least(character_id, "join_guild", 1).await?;
        }
        Ok(())
    }

    async fn set_progress_at_least(
        &self,
        character_id: i64,
        objective_kind: &str,
        value: i64,
    ) -> Result<(), sqlx::Error> {
        if value <= 0 {
            return Ok(());
        }
        let rows = progress_quests(&self.pool, objective_kind).await?;
        let mut tx = self.pool.begin().await?;
        for row in rows {
            ensure_progress_row(&mut tx, character_id, &row).await?;
            set_progress_row_at_least(&mut tx, character_id, &row, value).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

async fn progress_quests(pool: &PgPool, objective_kind: &str) -> Result<Vec<ProgressQuestRow>, sqlx::Error> {
    sqlx::query_as::<_, ProgressQuestRow>(
        r#"
        select
          id,
          greatest(coalesce((objectives->>'required')::bigint, 1), 1)::bigint as required,
          case when category = 'daily' then current_date::text else 'once' end as period_key
        from quest_templates
        where enabled = true and objectives->>'kind' = $1
        "#,
    )
    .bind(objective_kind)
    .fetch_all(pool)
    .await
}

async fn ensure_progress_row(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    row: &ProgressQuestRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        insert into character_quests (character_id, quest_id, period_key)
        values ($1, $2, $3)
        on conflict (character_id, quest_id, period_key) do nothing
        "#,
    )
    .bind(character_id)
    .bind(&row.id)
    .bind(&row.period_key)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn increment_progress_row(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    row: &ProgressQuestRow,
    amount: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        update character_quests
        set progress = jsonb_build_object(
              'value',
              least($4, coalesce((progress->>'value')::bigint, 0) + greatest(0, $5))
            ),
            completed_at = case
              when completed_at is not null then completed_at
              when least($4, coalesce((progress->>'value')::bigint, 0) + greatest(0, $5)) >= $4 then now()
              else completed_at
            end
        where character_id = $1 and quest_id = $2 and period_key = $3 and claimed_at is null
        "#,
    )
    .bind(character_id)
    .bind(&row.id)
    .bind(&row.period_key)
    .bind(row.required)
    .bind(amount)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn set_progress_row_at_least(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    row: &ProgressQuestRow,
    value: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        update character_quests
        set progress = jsonb_build_object(
              'value',
              least($4, greatest(coalesce((progress->>'value')::bigint, 0), $5))
            ),
            completed_at = case
              when completed_at is not null then completed_at
              when least($4, greatest(coalesce((progress->>'value')::bigint, 0), $5)) >= $4 then now()
              else completed_at
            end
        where character_id = $1 and quest_id = $2 and period_key = $3 and claimed_at is null
        "#,
    )
    .bind(character_id)
    .bind(&row.id)
    .bind(&row.period_key)
    .bind(row.required)
    .bind(value)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn sync_claimable_progress(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template: &QuestTemplateRow,
) -> Result<(), sqlx::Error> {
    let kind = template.objectives.get("kind").and_then(Value::as_str).unwrap_or_default();
    if kind == "level" {
        let (level,): (i32,) = sqlx::query_as("select level from characters where id = $1 and deleted_at is null")
            .bind(character_id)
            .fetch_one(&mut **tx)
            .await?;
        let row = ProgressQuestRow {
            id: template.id.clone(),
            required: template.required,
            period_key: template.period_key.clone(),
        };
        set_progress_row_at_least(tx, character_id, &row, i64::from(level)).await?;
    }
    Ok(())
}

async fn apply_rewards(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    rewards: &Value,
    summary: &mut QuestRewardSummary,
) -> Result<(), sqlx::Error> {
    let gold = rewards.get("gold").and_then(Value::as_i64).unwrap_or(0).clamp(0, 10_000);
    if gold > 0 {
        sqlx::query("update characters set gold = gold + $2 where id = $1 and deleted_at is null")
            .bind(character_id)
            .bind(gold)
            .execute(&mut **tx)
            .await?;
        summary.gold = gold;
    }

    if let Some(items) = rewards.get("items").and_then(Value::as_array) {
        for item in items {
            if let Ok(reward) = serde_json::from_value::<RewardItem>(item.clone()) {
                let quantity = reward.quantity.unwrap_or(1).clamp(1, 99);
                let bind = reward.bind.unwrap_or(true);
                if let Some(name) = grant_reward_item(tx, character_id, &reward.template_id, quantity, bind).await? {
                    summary.items.push(format!("{} x{}", name, quantity));
                }
            }
        }
    }
    Ok(())
}

async fn grant_reward_item(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
    bind: bool,
) -> Result<Option<String>, sqlx::Error> {
    let Some(template) = sqlx::query_as::<_, ItemTemplateRow>(
        "select name, stackable from item_templates where id = $1",
    )
    .bind(template_id)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(None);
    };

    if template.stackable {
        let updated = sqlx::query(
            r#"
            update inventory_items
            set quantity = quantity + $3
            where id = (
              select id
              from inventory_items
              where character_id = $1 and template_id = $2 and location = 'bag' and bind = $4
              order by id asc
              limit 1
            )
            "#,
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .bind(bind)
        .execute(&mut **tx)
        .await?;
        if updated.rows_affected() == 0 {
            sqlx::query(
                r#"
                insert into inventory_items (character_id, template_id, quantity, location, bind, extra)
                values ($1, $2, $3, 'bag', $4, $5)
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity)
            .bind(bind)
            .bind(json!({"source":"quest_reward"}))
            .execute(&mut **tx)
            .await?;
        }
    } else {
        for _ in 0..quantity {
            sqlx::query(
                r#"
                insert into inventory_items (character_id, template_id, quantity, location, bind, extra)
                values ($1, $2, 1, 'bag', $3, $4)
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(bind)
            .bind(json!({"source":"quest_reward"}))
            .execute(&mut **tx)
            .await?;
        }
    }

    Ok(Some(template.name))
}

fn quest_view(row: QuestViewRow) -> PlayerQuestView {
    let status = if row.claimed_at.is_some() {
        "claimed"
    } else if row.completed_at.is_some() || row.progress >= row.required {
        "completed"
    } else if row.progress > 0 {
        "progress"
    } else {
        "available"
    };
    PlayerQuestView {
        id: row.id,
        category: row.category,
        name: row.name,
        description: row.description,
        objectives: row.objectives,
        rewards: row.rewards,
        progress: row.progress.min(row.required),
        required: row.required,
        status: status.into(),
        sort_order: row.sort_order,
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RewardItem {
    template_id: String,
    quantity: Option<i64>,
    bind: Option<bool>,
}
