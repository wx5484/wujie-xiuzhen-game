use cq_protocol::dto::{AfkSettleResult, AfkStatusView};
use serde_json::json;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
struct AfkStatusRow {
    active: bool,
    mode: String,
    zone: Option<String>,
    room: Option<String>,
    training_skill_id: Option<String>,
    training_skill_name: Option<String>,
    started_at: Option<String>,
    last_settled_at: Option<String>,
    exp_per_minute: i64,
    gold_per_minute: i64,
}

#[derive(Debug, Clone, FromRow)]
struct AfkSettleSessionRow {
    mode: String,
    training_skill_name: Option<String>,
    ticks: i64,
}

#[derive(Debug, Clone, FromRow)]
struct AfkCharacterContextRow {
    level: i32,
    gold: i64,
}

#[derive(Debug, Clone)]
pub struct AfkRepository {
    pool: PgPool,
}

impl AfkRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn status(&self, character_id: i64) -> Result<AfkStatusView, sqlx::Error> {
        let row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            select
              active,
              coalesce(state->>'mode', 'safe') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            from afk_sessions
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(status_view).unwrap_or_else(|| AfkStatusView {
            active: false,
            mode: "safe".into(),
            zone: None,
            room: None,
            training_skill_id: None,
            training_skill_name: None,
            started_at: None,
            last_settled_at: None,
            exp_per_minute: 0,
            gold_per_minute: 0,
        }))
    }

    pub async fn start(
        &self,
        character_id: i64,
        _level: i32,
        training_skill_id: &str,
        training_skill_name: &str,
    ) -> Result<AfkStatusView, sqlx::Error> {
        let state = json!({
            "mode": "skill_study",
            "training_skill_id": training_skill_id,
            "training_skill_name": training_skill_name
        });
        let row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            insert into afk_sessions
              (character_id, active, started_at, last_settled_at, exp_per_minute, gold_per_minute, state)
            values ($1, true, now(), now(), $2, $3, $4)
            on conflict (character_id) do update set
              active = true,
              started_at = now(),
              last_settled_at = now(),
              exp_per_minute = excluded.exp_per_minute,
              gold_per_minute = excluded.gold_per_minute,
              state = excluded.state
            returning
              active,
              coalesce(state->>'mode', 'safe') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            "#,
        )
        .bind(character_id)
        .bind(0_i64)
        .bind(0_i64)
        .bind(state)
        .fetch_one(&self.pool)
        .await?;
        Ok(status_view(row))
    }

    pub async fn start_training(
        &self,
        character_id: i64,
        level: i32,
    ) -> Result<AfkStatusView, sqlx::Error> {
        let exp_per_tick = practice_exp_per_tick(&self.pool, character_id, level).await?;
        let state = json!({
            "mode": "practice",
            "training_skill_id": null,
            "training_skill_name": "炼狱修炼"
        });
        let row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            insert into afk_sessions
              (character_id, active, started_at, last_settled_at, exp_per_minute, gold_per_minute, state)
            values ($1, true, now(), now(), $2, 0, $3)
            on conflict (character_id) do update set
              active = true,
              started_at = now(),
              last_settled_at = now(),
              exp_per_minute = excluded.exp_per_minute,
              gold_per_minute = excluded.gold_per_minute,
              state = excluded.state
            returning
              active,
              coalesce(state->>'mode', 'practice') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            "#,
        )
        .bind(character_id)
        .bind(exp_per_tick.saturating_mul(12))
        .bind(state)
        .fetch_one(&self.pool)
        .await?;
        Ok(status_view(row))
    }

    pub async fn start_wild(
        &self,
        character_id: i64,
        level: i32,
        zone: &str,
        room: &str,
        exp_per_minute: i64,
        gold_per_minute: i64,
    ) -> Result<AfkStatusView, sqlx::Error> {
        let state = json!({
            "mode": "wild",
            "zone": zone,
            "room": room,
            "level": level.max(1)
        });
        let row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            insert into afk_sessions
              (character_id, active, started_at, last_settled_at, exp_per_minute, gold_per_minute, state)
            values ($1, true, now(), now(), $2, $3, $4)
            on conflict (character_id) do update set
              active = true,
              started_at = now(),
              last_settled_at = now(),
              exp_per_minute = excluded.exp_per_minute,
              gold_per_minute = excluded.gold_per_minute,
              state = excluded.state
            returning
              active,
              coalesce(state->>'mode', 'safe') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            "#,
        )
        .bind(character_id)
        .bind(exp_per_minute.max(1))
        .bind(gold_per_minute.max(0))
        .bind(state)
        .fetch_one(&self.pool)
        .await?;
        Ok(status_view(row))
    }

    pub async fn settle(&self, character_id: i64) -> Result<AfkSettleResult, sqlx::Error> {
        self.settle_with_cap(character_id, 120).await
    }

    pub async fn settle_with_cap(&self, character_id: i64, max_minutes: i64) -> Result<AfkSettleResult, sqlx::Error> {
        let max_ticks = max_minutes.clamp(1, 10_080).saturating_mul(12);
        let mut tx = self.pool.begin().await?;
        let session = sqlx::query_as::<_, AfkSettleSessionRow>(
            r#"
            select
              coalesce(state->>'mode', 'practice') as mode,
              state->>'training_skill_name' as training_skill_name,
              least($2, greatest(0, floor(extract(epoch from (now() - coalesce(last_settled_at, now()))) / 5)::bigint)) as ticks
            from afk_sessions
            where character_id = $1 and active = true
            for update
            "#,
        )
        .bind(character_id)
        .bind(max_ticks)
        .fetch_one(&mut *tx)
        .await?;

        let context = sqlx::query_as::<_, AfkCharacterContextRow>(
            r#"
            select level, gold
            from characters
            where id = $1 and deleted_at is null
            for update
            "#,
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        let (treasure_stage, cultivation_stage): (i64, i64) = sqlx::query_as(
            r#"
            select
              greatest(1, least(10, coalesce((select max(stage) from treasures where character_id = $1 and equipped = true), 0)))::bigint,
              greatest(1, least(9, coalesce((select max(layer) from cultivation_states where character_id = $1), 1)))::bigint
            "#,
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        let settled_ticks = session.ticks.min(context.gold / 10_000).max(0);
        let exp = if session.mode == "practice" {
            settled_ticks
                .saturating_mul(i64::from(context.level.max(1)))
                .saturating_mul(treasure_stage)
                .saturating_mul(cultivation_stage)
        } else {
            0
        };
        let gold = 0_i64;
        let should_stop = session.ticks > settled_ticks && session.ticks > 0;

        if settled_ticks > 0 {
            sqlx::query("update characters set gold = gold - $2 where id = $1")
                .bind(character_id)
                .bind(settled_ticks.saturating_mul(10_000))
                .execute(&mut *tx)
                .await?;
        }

        let status_row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            update afk_sessions
            set active = case when $2 then false else active end,
                last_settled_at = case when $3 then now() else last_settled_at end
            where character_id = $1
            returning
              active,
              coalesce(state->>'mode', 'practice') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            "#,
        )
        .bind(character_id)
        .bind(should_stop)
        .bind(settled_ticks > 0 || should_stop)
        .fetch_one(&mut *tx)
        .await?;

        if exp > 0 || gold > 0 || (session.mode == "skill_study" && settled_ticks > 0) {
            sqlx::query(
                r#"
                insert into afk_reward_logs (character_id, exp, gold, detail)
                values ($1, $2, $3, jsonb_build_object('mode', $4, 'minutes', $5))
                "#,
            )
            .bind(character_id)
            .bind(exp)
            .bind(gold)
            .bind(&session.mode)
            .bind(settled_ticks)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let skill_gain = if session.mode == "skill_study" { settled_ticks * 5 } else { 0 };
        Ok(AfkSettleResult {
            status: status_view(status_row),
            exp,
            gold,
            minutes: settled_ticks,
            message: if exp > 0 || gold > 0 || skill_gain > 0 {
                if session.mode == "practice" {
                    format!("炼狱修炼结算：完成 {} 次修炼，扣除金币 {}，获得 {} 经验。", settled_ticks, settled_ticks * 10_000, exp)
                } else {
                    format!(
                        "虚境研修结算：完成 {} 次研修，扣除金币 {}，{}技能经验 +{}。",
                        settled_ticks,
                        settled_ticks * 10_000,
                        session.training_skill_name.as_deref().unwrap_or("已选技能"),
                        skill_gain
                    )
                }
            } else if should_stop {
                "金币不足，打坐修炼已自动停止。".into()
            } else {
                "打坐时间不足 5 秒，暂未产生收益。".into()
            },
            adventure: None,
        })
    }

    pub async fn stop(&self, character_id: i64) -> Result<AfkStatusView, sqlx::Error> {
        let row = sqlx::query_as::<_, AfkStatusRow>(
            r#"
            update afk_sessions
            set active = false
            where character_id = $1
            returning
              active,
              coalesce(state->>'mode', 'safe') as mode,
              state->>'zone' as zone,
              state->>'room' as room,
              state->>'training_skill_id' as training_skill_id,
              state->>'training_skill_name' as training_skill_name,
              started_at::text as started_at,
              last_settled_at::text as last_settled_at,
              exp_per_minute,
              gold_per_minute
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(status_view(row))
    }
}

async fn practice_exp_per_tick(pool: &PgPool, character_id: i64, level: i32) -> Result<i64, sqlx::Error> {
    let (treasure_stage, cultivation_stage): (i64, i64) = sqlx::query_as(
        r#"
        select
          greatest(1, least(10, coalesce((select max(stage) from treasures where character_id = $1 and equipped = true), 0)))::bigint,
          greatest(1, least(9, coalesce((select layer from cultivation_states where character_id = $1), 1)))::bigint
        "#,
    )
    .bind(character_id)
    .fetch_one(pool)
    .await?;
    Ok(i64::from(level.max(1)).saturating_mul(treasure_stage).saturating_mul(cultivation_stage))
}

fn status_view(row: AfkStatusRow) -> AfkStatusView {
    AfkStatusView {
        active: row.active,
        mode: row.mode,
        zone: row.zone,
        room: row.room,
        training_skill_id: row.training_skill_id,
        training_skill_name: row.training_skill_name,
        started_at: row.started_at,
        last_settled_at: row.last_settled_at,
        exp_per_minute: row.exp_per_minute,
        gold_per_minute: row.gold_per_minute,
    }
}
