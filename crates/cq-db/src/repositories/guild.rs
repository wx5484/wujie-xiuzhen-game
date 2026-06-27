use std::collections::{HashMap, HashSet};

use cq_domain::inventory::bag_limit_for_level;
use rand::{thread_rng, Rng};
use serde::Serialize;
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use thiserror::Error;

const CREATE_GUILD_COST_GOLD: i64 = 10_000_000;
const GUILD_DONATION_GOLD_PER_POINT: i64 = 10_000;
const MAX_APPLICATION_MESSAGE_LEN: usize = 120;
const GUILD_MEMBER_CAP: i64 = 10;
const GUILD_MAX_LEVEL: i32 = 20;
const VIRTUAL_SABAK_GUILD: &str = "比奇远征队";
const GUILD_LEVEL_THRESHOLDS: [i64; 20] = [
    0, 1_000, 2_500, 4_500, 7_000, 10_000, 14_000, 19_000, 26_000, 36_500, 50_000, 68_000,
    90_000, 118_000, 152_000, 193_000, 242_000, 300_000, 368_000, 446_000,
];

#[derive(Debug, Clone, Serialize)]
pub struct GuildRecord {
    pub id: i64,
    pub name: String,
    pub notice: String,
    pub level: i32,
    pub funds: i64,
    pub sabak_owner: bool,
    pub member_count: i64,
    pub joined: bool,
    pub role: Option<String>,
    pub contribution: i64,
    pub pending_application: bool,
    pub projects: Vec<GuildProjectRecord>,
    pub totems: Vec<GuildTotemRecord>,
    pub war_techs: Vec<GuildWarTechRecord>,
    pub sabak_tax_claimed_today: bool,
}

#[derive(Debug, Clone, FromRow, Serialize)]
struct GuildRow {
    pub id: i64,
    pub name: String,
    pub notice: String,
    pub level: i32,
    pub funds: i64,
    pub sabak_owner: bool,
    pub member_count: i64,
    pub joined: bool,
    pub role: Option<String>,
    pub contribution: i64,
    pub pending_application: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuildProjectRecord {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub progress: i64,
    pub required: i64,
    pub completed: bool,
    pub completed_today: bool,
    pub min_level: i32,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuildTotemRecord {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub level: i32,
    pub next_cost: i64,
    pub max_level: i32,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuildWarTechRecord {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub level: i32,
    pub next_cost: i64,
    pub score_bonus: i64,
    pub unlocked: bool,
}

#[derive(Debug, Clone)]
pub struct GuildTaskOutcome {
    pub guild: GuildRecord,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GuildBenefitOutcome {
    pub guild: GuildRecord,
    pub message: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GuildApplicationRecord {
    pub id: i64,
    pub guild_id: i64,
    pub guild_name: String,
    pub character_id: i64,
    pub character_name: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GuildRepository {
    pool: PgPool,
}

#[derive(Debug, Error)]
pub enum GuildJoinError {
    #[error("guild not found")]
    NotFound,
    #[error("guild is full")]
    Full,
    #[error("character already joined another guild")]
    AlreadyInGuild,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum GuildActionError {
    #[error("guild not found")]
    NotFound,
    #[error("invalid name")]
    InvalidName,
    #[error("invalid amount")]
    InvalidAmount,
    #[error("invalid guild task")]
    InvalidTask,
    #[error("guild task already completed today")]
    AlreadyCompleted,
    #[error("guild benefit already claimed today")]
    AlreadyClaimed,
    #[error("guild task is unavailable")]
    TaskUnavailable,
    #[error("guild feature is locked")]
    FeatureLocked,
    #[error("max level reached")]
    MaxLevel,
    #[error("guild is full")]
    GuildFull,
    #[error("bag is full")]
    BagFull,
    #[error("not enough gold")]
    NotEnoughGold,
    #[error("not enough contribution")]
    NotEnoughContribution,
    #[error("not enough material")]
    NotEnoughMaterial,
    #[error("character already joined another guild")]
    AlreadyInGuild,
    #[error("application already pending")]
    AlreadyPending,
    #[error("permission denied")]
    PermissionDenied,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl GuildRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list(&self) -> Result<Vec<GuildRecord>, sqlx::Error> {
        self.list_for_character(None).await
    }

    pub async fn list_for_character(&self, character_id: Option<i64>) -> Result<Vec<GuildRecord>, sqlx::Error> {
        let _ = self.settle_sabak_if_due().await?;
        let rows = sqlx::query_as::<_, GuildRow>(
            r#"
            select
              g.id,
              g.name,
              g.notice,
              g.level,
              g.funds,
              g.sabak_owner,
              (select count(*)::bigint from guild_members gm where gm.guild_id = g.id) as member_count,
              exists(
                select 1 from guild_members mine where mine.guild_id = g.id and mine.character_id = $1
              ) as joined,
              (
                select mine.role from guild_members mine
                where mine.guild_id = g.id and mine.character_id = $1
                limit 1
              ) as role,
              coalesce((
                select mine.contribution from guild_members mine
                where mine.guild_id = g.id and mine.character_id = $1
                limit 1
              ), 0)::bigint as contribution,
              exists(
                select 1 from guild_applications ga
                where ga.guild_id = g.id and ga.character_id = $1 and ga.status = 'pending'
              ) as pending_application
            from guilds g
            order by g.level desc, g.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;

        let mut guilds = Vec::with_capacity(rows.len());
        for row in rows {
            let projects = self
                .projects_for_guild(row.id, character_id, row.level, row.member_count)
                .await?;
            let totems = self.totems_for_guild(row.id, character_id, row.level).await?;
            let war_techs = self.war_techs_for_guild(row.id, row.level).await?;
            let sabak_tax_claimed_today = self.sabak_tax_claimed_today(row.id, character_id).await?;
            guilds.push(row.into_record(projects, totems, war_techs, sabak_tax_claimed_today));
        }
        Ok(guilds)
    }

    pub async fn join(&self, character_id: i64, guild_id: i64) -> Result<GuildRecord, GuildJoinError> {
        let mut tx = self.pool.begin().await?;

        let guild_exists: Option<(i64,)> = sqlx::query_as("select id from guilds where id = $1 for update")
            .bind(guild_id)
            .fetch_optional(&mut *tx)
            .await?;
        if guild_exists.is_none() {
            return Err(GuildJoinError::NotFound);
        }
        let (member_count,): (i64,) =
            sqlx::query_as("select count(*)::bigint from guild_members where guild_id = $1")
                .bind(guild_id)
                .fetch_one(&mut *tx)
                .await?;

        let current: Option<(i64,)> =
            sqlx::query_as("select guild_id from guild_members where character_id = $1 limit 1")
                .bind(character_id)
                .fetch_optional(&mut *tx)
                .await?;

        match current {
            Some((current_guild_id,)) if current_guild_id == guild_id => {}
            Some(_) => return Err(GuildJoinError::AlreadyInGuild),
            None => {
                if member_count >= GUILD_MEMBER_CAP {
                    return Err(GuildJoinError::Full);
                }
                sqlx::query(
                    r#"
                    insert into guild_members (guild_id, character_id, role)
                    values ($1, $2, 'member')
                    "#,
                )
                .bind(guild_id)
                .bind(character_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        clear_pending_applications(&mut tx, character_id).await?;
        tx.commit().await?;
        self.get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildJoinError::Database)
    }

    pub async fn create(&self, character_id: i64, name: &str) -> Result<GuildRecord, GuildActionError> {
        let name = name.trim();
        if !valid_guild_name(name) {
            return Err(GuildActionError::InvalidName);
        }

        let mut tx = self.pool.begin().await?;
        let current: Option<(i64,)> =
            sqlx::query_as("select guild_id from guild_members where character_id = $1 limit 1")
                .bind(character_id)
                .fetch_optional(&mut *tx)
                .await?;
        if current.is_some() {
            return Err(GuildActionError::AlreadyInGuild);
        }

        let (gold,): (i64,) = sqlx::query_as(
            "select gold from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::NotFound)?;
        if gold < CREATE_GUILD_COST_GOLD {
            return Err(GuildActionError::NotEnoughGold);
        }

        let (guild_id,): (i64,) = sqlx::query_as(
            r#"
            insert into guilds (name, notice, funds)
            values ($1, '新行会成立，欢迎勇士加入。', 0)
            returning id
            "#,
        )
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("update characters set gold = gold - $2 where id = $1")
            .bind(character_id)
            .bind(CREATE_GUILD_COST_GOLD)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r#"
            insert into guild_members (guild_id, character_id, role, contribution)
            values ($1, $2, 'leader', 0)
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
        clear_pending_applications(&mut tx, character_id).await?;
        tx.commit().await?;
        self.get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)
    }

    pub async fn apply(
        &self,
        character_id: i64,
        guild_id: i64,
        message: &str,
    ) -> Result<GuildRecord, GuildActionError> {
        let mut tx = self.pool.begin().await?;
        let guild_exists: Option<(i64,)> = sqlx::query_as("select id from guilds where id = $1")
            .bind(guild_id)
            .fetch_optional(&mut *tx)
            .await?;
        if guild_exists.is_none() {
            return Err(GuildActionError::NotFound);
        }
        let current: Option<(i64,)> =
            sqlx::query_as("select guild_id from guild_members where character_id = $1 limit 1")
                .bind(character_id)
                .fetch_optional(&mut *tx)
                .await?;
        if current.is_some() {
            return Err(GuildActionError::AlreadyInGuild);
        }
        let pending: Option<(i64,)> = sqlx::query_as(
            "select id from guild_applications where guild_id = $1 and character_id = $2 and status = 'pending'",
        )
        .bind(guild_id)
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?;
        if pending.is_some() {
            return Err(GuildActionError::AlreadyPending);
        }
        let message = trim_message(message);
        sqlx::query(
            r#"
            insert into guild_applications (guild_id, character_id, message, status)
            values ($1, $2, $3, 'pending')
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .bind(message)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)
    }

    pub async fn donate(&self, character_id: i64, gold: i64) -> Result<GuildRecord, GuildActionError> {
        if gold <= 0 || gold % GUILD_DONATION_GOLD_PER_POINT != 0 {
            return Err(GuildActionError::InvalidAmount);
        }
        let points = gold / GUILD_DONATION_GOLD_PER_POINT;
        let mut tx = self.pool.begin().await?;
        let (guild_id,): (i64,) =
            sqlx::query_as("select guild_id from guild_members where character_id = $1 for update")
                .bind(character_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(GuildActionError::NotFound)?;
        let (current_gold,): (i64,) = sqlx::query_as(
            "select gold from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;
        if current_gold < gold {
            return Err(GuildActionError::NotEnoughGold);
        }
        sqlx::query("update characters set gold = gold - $2 where id = $1")
            .bind(character_id)
            .bind(gold)
            .execute(&mut *tx)
            .await?;
        add_guild_progress(&mut tx, guild_id, points).await?;
        sqlx::query(
            "update guild_members set contribution = contribution + $2 where guild_id = $1 and character_id = $3",
        )
        .bind(guild_id)
        .bind(points)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)
    }

    pub async fn complete_task(
        &self,
        character_id: i64,
        task_kind: &str,
    ) -> Result<GuildTaskOutcome, GuildActionError> {
        let task = guild_task_definition(task_kind).ok_or(GuildActionError::InvalidTask)?;
        let mut tx = self.pool.begin().await?;
        let context = sqlx::query_as::<_, GuildTaskContext>(
            r#"
            select
              gm.guild_id,
              g.level as guild_level,
              c.level as character_level,
              c.gold as character_gold,
              (select count(*)::bigint from guild_members where guild_id = gm.guild_id) as member_count
            from guild_members gm
            join guilds g on g.id = gm.guild_id
            join characters c on c.id = gm.character_id and c.deleted_at is null
            where gm.character_id = $1
            for update of gm, g, c
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::NotFound)?;

        if context.guild_level < task.min_guild_level || context.character_level < task.min_level {
            return Err(GuildActionError::TaskUnavailable);
        }
        if context.character_gold < task.gold_cost {
            return Err(GuildActionError::NotEnoughGold);
        }

        let inserted = sqlx::query(
            r#"
            insert into guild_task_claims
              (guild_id, character_id, task_kind, period_key, contribution, funds_added)
            values ($1, $2, $3, current_date::text, $4, $5)
            on conflict (guild_id, character_id, task_kind, period_key) do nothing
            "#,
        )
        .bind(context.guild_id)
        .bind(character_id)
        .bind(task.kind)
        .bind(task.contribution)
        .bind(task.funds_added)
        .execute(&mut *tx)
        .await?;
        if inserted.rows_affected() == 0 {
            return Err(GuildActionError::AlreadyCompleted);
        }

        if task.gold_cost > 0 {
            sqlx::query("update characters set gold = gold - $2 where id = $1")
                .bind(character_id)
                .bind(task.gold_cost)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query(
            "update guild_members set contribution = contribution + $3 where guild_id = $1 and character_id = $2",
        )
        .bind(context.guild_id)
        .bind(character_id)
        .bind(task.contribution)
        .execute(&mut *tx)
        .await?;

        add_guild_progress(&mut tx, context.guild_id, task.funds_added).await?;
        tx.commit().await?;

        let guild = self
            .get_for_character(context.guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        Ok(GuildTaskOutcome {
            guild,
            message: format!(
                "已完成行会任务：{}，行会建设 +{}，个人贡献 +{}。",
                task.name, task.funds_added, task.contribution
            ),
        })
    }

    pub async fn claim_benefit(&self, character_id: i64) -> Result<GuildBenefitOutcome, GuildActionError> {
        let mut tx = self.pool.begin().await?;
        let Some((guild_id, guild_level)): Option<(i64, i32)> = sqlx::query_as(
            r#"
            select gm.guild_id, g.level
            from guild_members gm
            join guilds g on g.id = gm.guild_id
            where gm.character_id = $1
            for update of gm, g
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Err(GuildActionError::NotFound);
        };

        let inserted = sqlx::query(
            r#"
            insert into guild_benefit_claims (guild_id, character_id, level, period_key)
            values ($1, $2, $3, current_date::text)
            on conflict (guild_id, character_id, period_key) do nothing
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .bind(guild_level.clamp(1, GUILD_MAX_LEVEL))
        .execute(&mut *tx)
        .await?;
        if inserted.rows_affected() == 0 {
            return Err(GuildActionError::AlreadyClaimed);
        }

        let rewards = guild_benefit_rewards(guild_level);
        for reward in rewards {
            grant_bound_stackable(&mut tx, character_id, reward.template_id, reward.quantity).await?;
        }
        tx.commit().await?;

        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        let reward_text = guild_benefit_rewards(guild_level)
            .into_iter()
            .map(|reward| format!("{} x{}", reward.name, reward.quantity))
            .collect::<Vec<_>>()
            .join("、");
        Ok(GuildBenefitOutcome {
            guild,
            message: format!("已领取 {} 级行会福利：{}。", guild_level.clamp(1, GUILD_MAX_LEVEL), reward_text),
        })
    }

    pub async fn use_merit_token(&self, character_id: i64) -> Result<GuildBenefitOutcome, GuildActionError> {
        let mut tx = self.pool.begin().await?;
        let (guild_id,): (i64,) =
            sqlx::query_as("select guild_id from guild_members where character_id = $1 for update")
                .bind(character_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(GuildActionError::NotFound)?;
        let (gold,): (i64,) = sqlx::query_as(
            "select gold from characters where id = $1 and deleted_at is null for update",
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;
        if gold < 50_000 {
            return Err(GuildActionError::NotEnoughGold);
        }
        consume_stackable_any_bind(&mut tx, character_id, "guild_merit_token", 1).await?;
        sqlx::query("update characters set gold = gold - 50000 where id = $1")
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("update guild_members set contribution = contribution + 10 where guild_id = $1 and character_id = $2")
            .bind(guild_id)
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        Ok(GuildBenefitOutcome {
            guild,
            message: "已使用行会功勋令，消耗 5 万金币，个人贡献 +10。".into(),
        })
    }

    pub async fn buy_shop_item(
        &self,
        character_id: i64,
        item_id: &str,
    ) -> Result<GuildBenefitOutcome, GuildActionError> {
        let (template_id, name, contribution_cost, gold_cost, quantity, stackable) = match item_id.trim() {
            "blood_shadow" => ("ring_blood_shadow", "血色幽影", 1_000_000, 0, 1, false),
            "big_taizi" => ("potion_big_taizi", "护脉丹", 80, 0, 1, true),
            "jiuzhuan" => ("potion_jiuzhuan", "九转还魂丹", 200, 0, 1, true),
            _ => return Err(GuildActionError::InvalidTask),
        };
        let mut tx = self.pool.begin().await?;
        let (guild_id, contribution, gold): (i64, i64, i64) = sqlx::query_as(
            r#"
            select gm.guild_id, gm.contribution, c.gold
            from guild_members gm
            join characters c on c.id = gm.character_id
            where gm.character_id = $1 and c.deleted_at is null
            for update of gm, c
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::NotFound)?;
        if contribution < contribution_cost {
            return Err(GuildActionError::NotEnoughContribution);
        }
        if gold < gold_cost {
            return Err(GuildActionError::NotEnoughGold);
        }
        sqlx::query(
            "update guild_members set contribution = contribution - $3 where guild_id = $1 and character_id = $2",
        )
        .bind(guild_id)
        .bind(character_id)
        .bind(contribution_cost)
        .execute(&mut *tx)
        .await?;
        if gold_cost > 0 {
            sqlx::query("update characters set gold = gold - $2 where id = $1")
                .bind(character_id)
                .bind(gold_cost)
                .execute(&mut *tx)
                .await?;
        }
        if stackable {
            grant_bound_stackable(&mut tx, character_id, template_id, quantity).await?;
        } else {
            ensure_bag_room_for_new_rows(&mut tx, character_id, 1).await?;
            sqlx::query(
                r#"
                insert into inventory_items (character_id, template_id, quantity, location, bind, extra)
                values ($1, $2, 1, 'bag', true, $3)
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(json!({"source":"guild_shop"}))
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        let gold_text = if gold_cost > 0 {
            format!("、金币 {}", gold_cost)
        } else {
            String::new()
        };
        Ok(GuildBenefitOutcome {
            guild,
            message: format!("行会商城购买 {} 成功，消耗贡献 {}{}。", name, contribution_cost, gold_text),
        })
    }

    pub async fn upgrade_totem(
        &self,
        character_id: i64,
        totem: &str,
    ) -> Result<GuildBenefitOutcome, GuildActionError> {
        let totem = guild_totem_definition(totem).ok_or(GuildActionError::InvalidTask)?;
        let mut tx = self.pool.begin().await?;
        let (guild_id, guild_level, contribution): (i64, i32, i64) = sqlx::query_as(
            r#"
            select gm.guild_id, g.level, gm.contribution
            from guild_members gm
            join guilds g on g.id = gm.guild_id
            where gm.character_id = $1
            for update of gm, g
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::NotFound)?;
        if guild_level < 10 {
            return Err(GuildActionError::FeatureLocked);
        }
        sqlx::query("delete from guild_totems where character_id = $1 and guild_id <> $2")
            .bind(character_id)
            .bind(guild_id)
            .execute(&mut *tx)
            .await?;
        let current_level = sqlx::query_as::<_, (Option<i32>,)>(
            "select max(level) from guild_totems where character_id = $1 and guild_id = $2 and totem = $3",
        )
        .bind(character_id)
        .bind(guild_id)
        .bind(totem.kind)
        .fetch_one(&mut *tx)
        .await?
        .0
        .unwrap_or_default()
        .clamp(0, 100);
        if current_level >= 100 {
            return Err(GuildActionError::MaxLevel);
        }
        let cost = guild_growth_cost(current_level);
        if contribution < cost {
            return Err(GuildActionError::NotEnoughContribution);
        }
        sqlx::query(
            "update guild_members set contribution = contribution - $3 where guild_id = $1 and character_id = $2",
        )
        .bind(guild_id)
        .bind(character_id)
        .bind(cost)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            insert into guild_totems (character_id, guild_id, totem, level)
            values ($1, $2, $3, $4)
            on conflict (character_id, totem) do update set
              guild_id = excluded.guild_id,
              level = excluded.level,
              updated_at = now()
            "#,
        )
        .bind(character_id)
        .bind(guild_id)
        .bind(totem.kind)
        .bind(current_level + 1)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        Ok(GuildBenefitOutcome {
            guild,
            message: format!("{}升级成功：{} -> {}，消耗贡献 {}。", totem.name, current_level, current_level + 1, cost),
        })
    }

    pub async fn charge_war_tech(
        &self,
        character_id: i64,
        kind: &str,
    ) -> Result<GuildBenefitOutcome, GuildActionError> {
        let tech = guild_war_tech_definition(kind).ok_or(GuildActionError::InvalidTask)?;
        let mut tx = self.pool.begin().await?;
        let (guild_id, guild_level, contribution): (i64, i32, i64) = sqlx::query_as(
            r#"
            select gm.guild_id, g.level, gm.contribution
            from guild_members gm
            join guilds g on g.id = gm.guild_id
            where gm.character_id = $1
            for update of gm, g
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::NotFound)?;
        if guild_level < GUILD_MAX_LEVEL {
            return Err(GuildActionError::FeatureLocked);
        }
        let current_level = sqlx::query_as::<_, (Option<i32>,)>(
            "select max(level) from guild_war_tech where guild_id = $1 and kind = $2",
        )
        .bind(guild_id)
        .bind(tech.kind)
        .fetch_one(&mut *tx)
        .await?
        .0
        .unwrap_or_default()
        .clamp(0, 100);
        if current_level >= 100 {
            return Err(GuildActionError::MaxLevel);
        }
        let cost = guild_growth_cost(current_level);
        if contribution < cost {
            return Err(GuildActionError::NotEnoughContribution);
        }
        sqlx::query(
            "update guild_members set contribution = contribution - $3 where guild_id = $1 and character_id = $2",
        )
        .bind(guild_id)
        .bind(character_id)
        .bind(cost)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            insert into guild_war_tech (guild_id, kind, level, charged_points)
            values ($1, $2, $3, $4)
            on conflict (guild_id, kind) do update set
              level = excluded.level,
              charged_points = guild_war_tech.charged_points + excluded.charged_points,
              updated_at = now()
            "#,
        )
        .bind(guild_id)
        .bind(tech.kind)
        .bind(current_level + 1)
        .bind(cost)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        Ok(GuildBenefitOutcome {
            guild,
            message: format!("{}充能成功：{} -> {}，消耗贡献 {}。", tech.name, current_level, current_level + 1, cost),
        })
    }

    pub async fn claim_sabak_tax(&self, character_id: i64) -> Result<GuildBenefitOutcome, GuildActionError> {
        let mut tx = self.pool.begin().await?;
        let (guild_id, guild_name): (i64, String) = sqlx::query_as(
            r#"
            select g.id, g.name
            from guild_members gm
            join guilds g on g.id = gm.guild_id
            where gm.character_id = $1 and g.sabak_owner = true
            for update of g
            "#,
        )
        .bind(character_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(GuildActionError::FeatureLocked)?;
        let inserted = sqlx::query(
            r#"
            insert into guild_sabak_tax_claims (guild_id, character_id, period_key)
            values ($1, $2, current_date::text)
            on conflict (guild_id, character_id, period_key) do nothing
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
        if inserted.rows_affected() == 0 {
            return Err(GuildActionError::AlreadyClaimed);
        }
        sqlx::query("update characters set yuanbao = yuanbao + 100 where id = $1 and deleted_at is null")
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
        grant_bound_stackable(&mut tx, character_id, "treasure_shard", 5).await?;
        grant_bound_stackable(&mut tx, character_id, "cultivation_pill", 5).await?;
        grant_bound_stackable(&mut tx, character_id, "pet_food", 5).await?;
        tx.commit().await?;
        let guild = self
            .get_for_character(guild_id, Some(character_id))
            .await
            .map_err(GuildActionError::Database)?;
        Ok(GuildBenefitOutcome {
            guild,
            message: format!("已领取 {} 的沙城税收：元宝 +100，法宝碎片/修炼丹/灵兽粮各 x5。", guild_name),
        })
    }

    pub async fn applications_for_reviewer(
        &self,
        character_id: i64,
    ) -> Result<Vec<GuildApplicationRecord>, GuildActionError> {
        let guild_id = reviewer_guild_id(&self.pool, character_id)
            .await?
            .ok_or(GuildActionError::PermissionDenied)?;
        let rows = sqlx::query_as::<_, GuildApplicationRecord>(
            r#"
            select
              ga.id,
              ga.guild_id,
              g.name as guild_name,
              ga.character_id,
              c.name as character_name,
              ga.message,
              ga.status,
              ga.created_at::text as created_at
            from guild_applications ga
            join guilds g on g.id = ga.guild_id
            join characters c on c.id = ga.character_id
            where ga.guild_id = $1 and ga.status = 'pending'
            order by ga.created_at asc, ga.id asc
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn review_application(
        &self,
        reviewer_character_id: i64,
        application_id: i64,
        accept: bool,
    ) -> Result<Vec<GuildApplicationRecord>, GuildActionError> {
        let mut tx = self.pool.begin().await?;
        let Some((guild_id,)): Option<(i64,)> = sqlx::query_as(
            r#"
            select guild_id
            from guild_members
            where character_id = $1 and role in ('leader', 'elder')
            for update
            "#,
        )
        .bind(reviewer_character_id)
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Err(GuildActionError::PermissionDenied);
        };

        let Some((applicant_id,)): Option<(i64,)> = sqlx::query_as(
            "select character_id from guild_applications where id = $1 and guild_id = $2 and status = 'pending' for update",
        )
        .bind(application_id)
        .bind(guild_id)
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Err(GuildActionError::NotFound);
        };

        if accept {
            let current: Option<(i64,)> =
                sqlx::query_as("select guild_id from guild_members where character_id = $1 limit 1")
                    .bind(applicant_id)
                    .fetch_optional(&mut *tx)
                    .await?;
            if current.is_some() {
                return Err(GuildActionError::AlreadyInGuild);
            }
            let (member_count,): (i64,) =
                sqlx::query_as("select count(*)::bigint from guild_members where guild_id = $1")
                    .bind(guild_id)
                    .fetch_one(&mut *tx)
                    .await?;
            if member_count >= GUILD_MEMBER_CAP {
                return Err(GuildActionError::GuildFull);
            }
            sqlx::query(
                r#"
                insert into guild_members (guild_id, character_id, role)
                values ($1, $2, 'member')
                "#,
            )
            .bind(guild_id)
            .bind(applicant_id)
            .execute(&mut *tx)
            .await?;
            clear_other_pending_applications(&mut tx, applicant_id, application_id).await?;
        }

        let status = if accept { "accepted" } else { "rejected" };
        sqlx::query("update guild_applications set status = $2 where id = $1")
            .bind(application_id)
            .bind(status)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        self.applications_for_reviewer(reviewer_character_id).await
    }

    async fn get_for_character(
        &self,
        guild_id: i64,
        character_id: Option<i64>,
    ) -> Result<GuildRecord, sqlx::Error> {
        let row = sqlx::query_as::<_, GuildRow>(
            r#"
            select
              g.id,
              g.name,
              g.notice,
              g.level,
              g.funds,
              g.sabak_owner,
              (select count(*)::bigint from guild_members gm where gm.guild_id = g.id) as member_count,
              exists(
                select 1 from guild_members mine where mine.guild_id = g.id and mine.character_id = $2
              ) as joined,
              (
                select mine.role from guild_members mine
                where mine.guild_id = g.id and mine.character_id = $2
                limit 1
              ) as role,
              coalesce((
                select mine.contribution from guild_members mine
                where mine.guild_id = g.id and mine.character_id = $2
                limit 1
              ), 0)::bigint as contribution,
              exists(
                select 1 from guild_applications ga
                where ga.guild_id = g.id and ga.character_id = $2 and ga.status = 'pending'
              ) as pending_application
            from guilds g
            where g.id = $1
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        let projects = self
            .projects_for_guild(row.id, character_id, row.level, row.member_count)
            .await?;
        let totems = self.totems_for_guild(row.id, character_id, row.level).await?;
        let war_techs = self.war_techs_for_guild(row.id, row.level).await?;
        let sabak_tax_claimed_today = self.sabak_tax_claimed_today(row.id, character_id).await?;
        Ok(row.into_record(projects, totems, war_techs, sabak_tax_claimed_today))
    }

    async fn projects_for_guild(
        &self,
        guild_id: i64,
        character_id: Option<i64>,
        guild_level: i32,
        member_count: i64,
    ) -> Result<Vec<GuildProjectRecord>, sqlx::Error> {
        let progress_rows = sqlx::query_as::<_, GuildTaskProgressRow>(
            r#"
            select task_kind, count(*)::bigint as progress
            from guild_task_claims
            where guild_id = $1 and period_key = current_date::text
            group by task_kind
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;
        let progress = progress_rows
            .into_iter()
            .map(|row| (row.task_kind, row.progress))
            .collect::<HashMap<_, _>>();

        let completed_rows = sqlx::query_as::<_, GuildTaskKindRow>(
            r#"
            select task_kind
            from guild_task_claims
            where guild_id = $1
              and character_id = $2
              and period_key = current_date::text
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .fetch_all(&self.pool)
        .await?;
        let completed_today = completed_rows
            .into_iter()
            .map(|row| row.task_kind)
            .collect::<HashSet<_>>();

        Ok(guild_task_definitions()
            .into_iter()
            .map(|task| {
                let required = guild_task_required(task, guild_level, member_count);
                let current = progress.get(task.kind).copied().unwrap_or_default().min(required);
                GuildProjectRecord {
                    kind: task.kind.into(),
                    name: task.name.into(),
                    description: task.description.into(),
                    progress: current,
                    required,
                    completed: current >= required,
                    completed_today: completed_today.contains(task.kind),
                    min_level: task.min_level,
                    available: guild_level >= task.min_guild_level,
                }
            })
            .collect())
    }

    async fn totems_for_guild(
        &self,
        guild_id: i64,
        character_id: Option<i64>,
        guild_level: i32,
    ) -> Result<Vec<GuildTotemRecord>, sqlx::Error> {
        let mut levels = HashMap::<String, i32>::new();
        if let Some(character_id) = character_id {
            let rows = sqlx::query_as::<_, GuildTotemLevelRow>(
                r#"
                select totem, level
                from guild_totems
                where guild_id = $1 and character_id = $2
                "#,
            )
            .bind(guild_id)
            .bind(character_id)
            .fetch_all(&self.pool)
            .await?;
            levels.extend(rows.into_iter().map(|row| (row.totem, row.level)));
        }
        Ok(guild_totem_definitions()
            .into_iter()
            .map(|def| {
                let level = levels.get(def.kind).copied().unwrap_or_default().clamp(0, 100);
                GuildTotemRecord {
                    kind: def.kind.into(),
                    name: def.name.into(),
                    description: def.description.into(),
                    level,
                    next_cost: guild_growth_cost(level),
                    max_level: 100,
                    unlocked: guild_level >= 10,
                }
            })
            .collect())
    }

    async fn war_techs_for_guild(
        &self,
        guild_id: i64,
        guild_level: i32,
    ) -> Result<Vec<GuildWarTechRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, GuildWarTechLevelRow>(
            r#"
            select kind, level, charged_points
            from guild_war_tech
            where guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;
        let levels = rows
            .into_iter()
            .map(|row| (row.kind, (row.level, row.charged_points)))
            .collect::<HashMap<_, _>>();
        Ok(guild_war_tech_definitions()
            .into_iter()
            .map(|def| {
                let (level, charged_points) = levels.get(def.kind).copied().unwrap_or_default();
                let level = level.clamp(0, 100);
                GuildWarTechRecord {
                    kind: def.kind.into(),
                    name: def.name.into(),
                    description: def.description.into(),
                    level,
                    next_cost: guild_growth_cost(level),
                    score_bonus: guild_war_tech_score(level, charged_points),
                    unlocked: guild_level >= GUILD_MAX_LEVEL,
                }
            })
            .collect())
    }

    async fn sabak_tax_claimed_today(
        &self,
        guild_id: i64,
        character_id: Option<i64>,
    ) -> Result<bool, sqlx::Error> {
        let Some(character_id) = character_id else {
            return Ok(false);
        };
        let (claimed,): (bool,) = sqlx::query_as(
            r#"
            select exists(
              select 1
              from guild_sabak_tax_claims
              where guild_id = $1 and character_id = $2 and period_key = current_date::text
            )
            "#,
        )
        .bind(guild_id)
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(claimed)
    }

    pub async fn sabak_owner_name(&self) -> Result<String, sqlx::Error> {
        let _ = self.settle_sabak_if_due().await?;
        let (name,): (String,) = sqlx::query_as(
            r#"
            select coalesce(
              (select name from guilds where sabak_owner = true order by id asc limit 1),
              (select winner_name from guild_sabak_state where id = 1),
              $1
            )
            "#,
        )
        .bind(VIRTUAL_SABAK_GUILD)
        .fetch_one(&self.pool)
        .await?;
        Ok(name)
    }

    pub async fn settle_sabak_if_due(&self) -> Result<Option<String>, sqlx::Error> {
        let (should_settle,): (bool,) = sqlx::query_as(
            r#"
            with schedule as (
              select (date_trunc('week', now() at time zone 'Asia/Shanghai') + interval '6 days 20 hours') at time zone 'Asia/Shanghai' as battle_at
            )
            select now() >= battle_at
              and coalesce(
                (select last_settled_at < battle_at from guild_sabak_state where id = 1),
                true
              )
            from schedule
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        if !should_settle {
            return Ok(None);
        }

        let (top_avg_power,): (i64,) = sqlx::query_as(
            r#"
            select coalesce(avg(power), 0)::bigint
            from (
              select power
              from characters
              where deleted_at is null
              order by power desc, level desc, exp desc
              limit 10
            ) top_players
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let virtual_score = top_avg_power
            .saturating_mul(125)
            .saturating_mul(GUILD_MEMBER_CAP)
            / 100;

        let candidates = sqlx::query_as::<_, SabakCandidateRow>(
            r#"
            select
              g.id,
              g.name,
              coalesce(sum(c.power), 0)::bigint as member_power,
              coalesce((
                select sum(level::bigint * 100000 + charged_points * 10)
                from guild_war_tech
                where guild_id = g.id
              ), 0)::bigint as tech_power
            from guilds g
            join guild_members gm on gm.guild_id = g.id
            join characters c on c.id = gm.character_id and c.deleted_at is null
            where g.level >= $1
            group by g.id, g.name
            "#,
        )
        .bind(GUILD_MAX_LEVEL)
        .fetch_all(&self.pool)
        .await?;

        let mut best_name = VIRTUAL_SABAK_GUILD.to_string();
        let mut best_guild_id = None;
        let mut best_score = virtual_score;
        for candidate in candidates {
            let base = candidate.member_power.saturating_add(candidate.tech_power);
            let score = base.saturating_mul(sabak_score_multiplier()) / 100;
            if score > best_score {
                best_score = score;
                best_name = candidate.name;
                best_guild_id = Some(candidate.id);
            }
        }

        let mut tx = self.pool.begin().await?;
        sqlx::query("update guilds set sabak_owner = false")
            .execute(&mut *tx)
            .await?;
        if let Some(guild_id) = best_guild_id {
            sqlx::query("update guilds set sabak_owner = true where id = $1")
                .bind(guild_id)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query(
            r#"
            insert into guild_sabak_state (id, winner_guild_id, winner_name, last_settled_at, updated_at)
            values (1, $1, $2, now(), now())
            on conflict (id) do update set
              winner_guild_id = excluded.winner_guild_id,
              winner_name = excluded.winner_name,
              last_settled_at = excluded.last_settled_at,
              updated_at = now()
            "#,
        )
        .bind(best_guild_id)
        .bind(&best_name)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(Some(format!("沙城争霸结算完成：{} 以 {} 积分占领沙巴克。", best_name, best_score)))
    }
}

fn sabak_score_multiplier() -> i64 {
    thread_rng().gen_range(90_i64..=110_i64)
}

impl GuildRow {
    fn into_record(
        self,
        projects: Vec<GuildProjectRecord>,
        totems: Vec<GuildTotemRecord>,
        war_techs: Vec<GuildWarTechRecord>,
        sabak_tax_claimed_today: bool,
    ) -> GuildRecord {
        GuildRecord {
            id: self.id,
            name: self.name,
            notice: self.notice,
            level: self.level,
            funds: self.funds,
            sabak_owner: self.sabak_owner,
            member_count: self.member_count,
            joined: self.joined,
            role: self.role,
            contribution: self.contribution,
            pending_application: self.pending_application,
            projects,
            totems,
            war_techs,
            sabak_tax_claimed_today,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
struct GuildTaskContext {
    guild_id: i64,
    guild_level: i32,
    character_level: i32,
    character_gold: i64,
}

#[derive(Debug, Clone, FromRow)]
struct GuildTaskProgressRow {
    task_kind: String,
    progress: i64,
}

#[derive(Debug, Clone, FromRow)]
struct GuildTaskKindRow {
    task_kind: String,
}

#[derive(Debug, Clone, FromRow)]
struct GuildTotemLevelRow {
    totem: String,
    level: i32,
}

#[derive(Debug, Clone, FromRow)]
struct GuildWarTechLevelRow {
    kind: String,
    level: i32,
    charged_points: i64,
}

#[derive(Debug, Clone, FromRow)]
struct SabakCandidateRow {
    id: i64,
    name: String,
    member_power: i64,
    tech_power: i64,
}

#[derive(Debug, Clone, Copy)]
struct GuildTaskDefinition {
    kind: &'static str,
    name: &'static str,
    description: &'static str,
    min_level: i32,
    min_guild_level: i32,
    gold_cost: i64,
    funds_added: i64,
    contribution: i64,
}

#[derive(Debug, Clone, Copy)]
struct GuildTotemDefinition {
    kind: &'static str,
    name: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct GuildWarTechDefinition {
    kind: &'static str,
    name: &'static str,
    description: &'static str,
}

fn guild_task_definitions() -> Vec<GuildTaskDefinition> {
    vec![
        GuildTaskDefinition {
            kind: "patrol",
            name: "行会巡猎",
            description: "完成一次行会巡逻，推进今日行会活跃目标。",
            min_level: 1,
            min_guild_level: 1,
            gold_cost: 0,
            funds_added: 45,
            contribution: 45,
        },
        GuildTaskDefinition {
            kind: "supply",
            name: "补给建设",
            description: "消耗 1 万金币补充行会物资，稳定推进建设进度。",
            min_level: 1,
            min_guild_level: 1,
            gold_cost: 10_000,
            funds_added: 35,
            contribution: 35,
        },
        GuildTaskDefinition {
            kind: "boss_drill",
            name: "首领演武",
            description: "20 级后参与首领演武，为中后期 Boss 推进做准备。",
            min_level: 20,
            min_guild_level: 2,
            gold_cost: 0,
            funds_added: 20,
            contribution: 20,
        },
    ]
}

fn guild_totem_definitions() -> Vec<GuildTotemDefinition> {
    vec![
        GuildTotemDefinition {
            kind: "qiongqi",
            name: "穷奇图腾",
            description: "每级物理攻击 +100、魔法攻击 +100，仅对本人角色生效。",
        },
        GuildTotemDefinition {
            kind: "bifang",
            name: "毕方图腾",
            description: "每级暴击率 +0.2%、暴击伤害 +1%，仅对本人角色生效。",
        },
        GuildTotemDefinition {
            kind: "chenghuang",
            name: "乘黄图腾",
            description: "每级最大生命 +500、最大魔法 +500，仅对本人角色生效。",
        },
        GuildTotemDefinition {
            kind: "xuangui",
            name: "旋龟图腾",
            description: "每级物理防御 +50、魔法防御 +50，仅对本人角色生效。",
        },
    ]
}

fn guild_totem_definition(kind: &str) -> Option<GuildTotemDefinition> {
    let kind = kind.trim();
    guild_totem_definitions()
        .into_iter()
        .find(|definition| definition.kind == kind)
}

fn guild_war_tech_definitions() -> Vec<GuildWarTechDefinition> {
    vec![
        GuildWarTechDefinition {
            kind: "siege_chariot",
            name: "攻城战车",
            description: "20级行会攻沙科技，提升沙城争霸全局积分。",
        },
        GuildWarTechDefinition {
            kind: "defense_barrier",
            name: "守城结界",
            description: "20级行会守城科技，提升沙城争霸全局积分。",
        },
    ]
}

fn guild_war_tech_definition(kind: &str) -> Option<GuildWarTechDefinition> {
    let kind = kind.trim();
    guild_war_tech_definitions()
        .into_iter()
        .find(|definition| definition.kind == kind)
}

fn guild_growth_cost(current_level: i32) -> i64 {
    1_000 + i64::from(current_level.max(0)).pow(2) * 30
}

fn guild_war_tech_score(level: i32, charged_points: i64) -> i64 {
    i64::from(level.max(0)).saturating_mul(100_000) + charged_points.max(0).saturating_mul(10)
}

fn guild_task_definition(kind: &str) -> Option<GuildTaskDefinition> {
    let kind = kind.trim();
    guild_task_definitions()
        .into_iter()
        .find(|task| task.kind == kind)
}

fn guild_task_required(task: GuildTaskDefinition, guild_level: i32, member_count: i64) -> i64 {
    let level = i64::from(guild_level.max(1));
    match task.kind {
        "supply" => (member_count + level / 2).clamp(1, 20),
        "boss_drill" => ((member_count + 1) / 2).clamp(1, 12),
        _ => member_count.clamp(1, 30),
    }
}

async fn add_guild_progress(
    tx: &mut Transaction<'_, Postgres>,
    guild_id: i64,
    points: i64,
) -> Result<(), sqlx::Error> {
    if points <= 0 {
        return Ok(());
    }
    let (funds, current_level): (i64, i32) = sqlx::query_as(
        "update guilds set funds = funds + $2 where id = $1 returning funds, level",
    )
    .bind(guild_id)
    .bind(points)
    .fetch_one(&mut **tx)
    .await?;
    let next_level = guild_level_for_funds(funds).max(current_level).clamp(1, GUILD_MAX_LEVEL);
    if next_level != current_level {
        sqlx::query("update guilds set level = $2 where id = $1")
            .bind(guild_id)
            .bind(next_level)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

fn guild_level_for_funds(funds: i64) -> i32 {
    let funds = funds.max(0);
    GUILD_LEVEL_THRESHOLDS
        .iter()
        .position(|required| funds < *required)
        .map(|index| index as i32)
        .unwrap_or(GUILD_MAX_LEVEL)
        .max(1)
}

async fn clear_pending_applications(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("delete from guild_applications where character_id = $1 and status = 'pending'")
        .bind(character_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn clear_other_pending_applications(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    keep_application_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "delete from guild_applications where character_id = $1 and status = 'pending' and id <> $2",
    )
    .bind(character_id)
    .bind(keep_application_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct GuildBenefitReward {
    template_id: &'static str,
    name: &'static str,
    quantity: i64,
}

fn guild_benefit_rewards(guild_level: i32) -> Vec<GuildBenefitReward> {
    let quantity = i64::from(guild_level.clamp(1, GUILD_MAX_LEVEL));
    let skill_pages = quantity.saturating_mul(10);
    vec![
        benefit_reward("pet_food", "灵兽粮", quantity),
        benefit_reward("treasure_shard", "法宝碎片", quantity),
        benefit_reward("cultivation_pill", "修炼丹", quantity),
        benefit_reward("pill_insight", "悟性丹", quantity),
        benefit_reward("skill_page", "技能书残页", skill_pages),
        benefit_reward("stone_hongmeng", "鸿蒙石", quantity),
    ]
}

fn benefit_reward(template_id: &'static str, name: &'static str, quantity: i64) -> GuildBenefitReward {
    GuildBenefitReward { template_id, name, quantity }
}

async fn grant_bound_stackable(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), GuildActionError> {
    if quantity <= 0 {
        return Ok(());
    }
    let exists: Option<(bool,)> = sqlx::query_as("select stackable from item_templates where id = $1")
        .bind(template_id)
        .fetch_optional(&mut **tx)
        .await?;
    if exists.is_none() {
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
            and location = 'bag'
            and bind = true
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
        ensure_bag_room_for_new_rows(tx, character_id, 1).await?;
        sqlx::query(
            r#"
            insert into inventory_items (character_id, template_id, quantity, location, bind, extra)
            values ($1, $2, $3, 'bag', true, $4)
            "#,
        )
        .bind(character_id)
        .bind(template_id)
        .bind(quantity)
        .bind(json!({"source":"guild_benefit"}))
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn ensure_bag_room_for_new_rows(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    rows_needed: i64,
) -> Result<(), GuildActionError> {
    if rows_needed <= 0 {
        return Ok(());
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
    let limit = bag_limit_for_level(level) as i64;
    if bag_count.saturating_add(rows_needed) > limit {
        return Err(GuildActionError::BagFull);
    }
    Ok(())
}

async fn consume_stackable_any_bind(
    tx: &mut Transaction<'_, Postgres>,
    character_id: i64,
    template_id: &str,
    quantity: i64,
) -> Result<(), GuildActionError> {
    let rows = sqlx::query_as::<_, (i64, i64)>(
        r#"
        select id, quantity
        from inventory_items
        where character_id = $1
          and template_id = $2
          and location = 'bag'
        order by bind desc, id asc
        for update
        "#,
    )
    .bind(character_id)
    .bind(template_id)
    .fetch_all(&mut **tx)
    .await?;
    let available = rows.iter().map(|(_, qty)| *qty).sum::<i64>();
    if available < quantity.max(1) {
        return Err(GuildActionError::NotEnoughMaterial);
    }
    let mut remaining = quantity.max(1);
    for (id, qty) in rows {
        if remaining <= 0 {
            break;
        }
        if qty <= remaining {
            sqlx::query("delete from inventory_items where id = $1")
                .bind(id)
                .execute(&mut **tx)
                .await?;
            remaining -= qty;
        } else {
            sqlx::query("update inventory_items set quantity = quantity - $2 where id = $1")
                .bind(id)
                .bind(remaining)
                .execute(&mut **tx)
                .await?;
            remaining = 0;
        }
    }
    Ok(())
}

async fn reviewer_guild_id(pool: &PgPool, character_id: i64) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "select guild_id from guild_members where character_id = $1 and role in ('leader', 'elder') limit 1",
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|item| item.0))
}

fn valid_guild_name(name: &str) -> bool {
    let count = name.chars().count();
    (2..=16).contains(&count)
}

fn trim_message(message: &str) -> String {
    message
        .trim()
        .chars()
        .take(MAX_APPLICATION_MESSAGE_LEN)
        .collect::<String>()
}
