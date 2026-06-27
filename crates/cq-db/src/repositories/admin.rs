use cq_domain::character::{exp_for_level, initial_stats, power_from_stats, CharacterClass};
use cq_protocol::dto::{
    AdminAccountList, AdminAccountView, AdminAuditLogList, AdminAuditLogView,
    AdminCharacterDetail, AdminCharacterList, AdminCharacterListItem, AdminItemTemplateInput, AdminItemTemplateList,
    AdminItemTemplateView, AdminMailOverview, AdminMailView, AdminMobTemplateList,
    AdminMobTemplateView,
    DashboardSummary,
};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;

use super::{
    character::{state_view, stats_view, CharacterStateRecord, CharacterStatsRecord},
    inventory::InventoryRepository,
};

#[derive(Debug, Clone)]
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn dashboard(&self) -> Result<DashboardSummary, sqlx::Error> {
        let online: (i64,) = sqlx::query_as("select count(*) from character_state where online = true")
            .fetch_one(&self.pool)
            .await?;
        let accounts: (i64,) = sqlx::query_as("select count(*) from accounts").fetch_one(&self.pool).await?;
        let characters: (i64,) =
            sqlx::query_as("select count(*) from characters where deleted_at is null").fetch_one(&self.pool).await?;
        let mails: (i64,) = sqlx::query_as("select count(*) from mails").fetch_one(&self.pool).await?;
        let guilds: (i64,) = sqlx::query_as("select count(*) from guilds").fetch_one(&self.pool).await?;
        let pending_backups: (i64,) =
            sqlx::query_as("select count(*) from backup_jobs where status in ('queued','running')")
                .fetch_one(&self.pool)
                .await?;
        Ok(DashboardSummary {
            online: online.0,
            accounts: accounts.0,
            characters: characters.0,
            mails: mails.0,
            guilds: guilds.0,
            pending_backups: pending_backups.0,
        })
    }

    pub async fn accounts(&self) -> Result<AdminAccountList, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("select count(*) from accounts")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, AccountAdminRow>(
            r#"
            select
              a.id,
              a.username,
              a.email,
              a.status,
              a.created_at,
              max(s.created_at) as last_login_at,
              count(distinct c.id) filter (where c.deleted_at is null) as character_count
            from accounts a
            left join sessions s on s.account_id = a.id
            left join characters c on c.account_id = a.id
            group by a.id, a.username, a.email, a.status, a.created_at
            order by a.created_at desc
            limit 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminAccountList {
            total: total.0,
            accounts: rows
                .into_iter()
                .map(|row| AdminAccountView {
                    id: row.id,
                    username: row.username,
                    email: row.email,
                    status: row.status,
                    created_at: row.created_at.to_string(),
                    last_login_at: row.last_login_at.map(|value| value.to_string()),
                    character_count: row.character_count,
                })
                .collect(),
        })
    }

    pub async fn characters(&self) -> Result<AdminCharacterList, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("select count(*) from characters where deleted_at is null")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, CharacterAdminRow>(
            r#"
            select
              c.id,
              c.account_id,
              a.username as account_username,
              c.name,
              c.class,
              c.level,
              c.exp,
              c.gold,
              c.yuanbao,
              c.power,
              cs.zone,
              cs.room,
              cs.hp,
              cs.mp,
              coalesce(cs.online, false) as online,
              c.created_at,
              c.updated_at
            from characters c
            join accounts a on a.id = c.account_id
            left join character_state cs on cs.character_id = c.id
            where c.deleted_at is null
            order by c.created_at desc
            limit 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminCharacterList {
            total: total.0,
            characters: rows
                .into_iter()
                .map(character_admin_item)
                .collect(),
        })
    }

    async fn character_admin_row(&self, character_id: i64) -> Result<CharacterAdminRow, sqlx::Error> {
        sqlx::query_as::<_, CharacterAdminRow>(
            r#"
            select
              c.id,
              c.account_id,
              a.username as account_username,
              c.name,
              c.class,
              c.level,
              c.exp,
              c.gold,
              c.yuanbao,
              c.power,
              cs.zone,
              cs.room,
              cs.hp,
              cs.mp,
              coalesce(cs.online, false) as online,
              c.created_at,
              c.updated_at
            from characters c
            join accounts a on a.id = c.account_id
            left join character_state cs on cs.character_id = c.id
            where c.deleted_at is null and c.id = $1
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn character_detail(&self, character_id: i64) -> Result<AdminCharacterDetail, sqlx::Error> {
        let row = self.character_admin_row(character_id).await?;
        let level = row.level;
        let stats = sqlx::query_as::<_, CharacterStatsRecord>(
            r#"
            select
              character_id,
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
        let state = sqlx::query_as::<_, CharacterStateRecord>(
            r#"
            select
              character_id,
              zone,
              room,
              hp,
              mp,
              stamina,
              stamina_max,
              online,
              pk_enabled,
              sweep_attack_players,
              updated_at::text as updated_at
            from character_state
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;
        let inventory = InventoryRepository::new(&self.pool).view(character_id, level).await?;
        Ok(AdminCharacterDetail {
            character: character_admin_item(row),
            stats: stats_view(stats),
            state: state_view(state),
            inventory,
        })
    }

    pub async fn mail_overview(&self) -> Result<AdminMailOverview, sqlx::Error> {
        let summary = sqlx::query_as::<_, MailSummaryRow>(
            r#"
            select
              count(*) as total,
              count(*) filter (where read = false) as unread,
              count(*) filter (where claimed = false) as unclaimed
            from mails
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let with_attachments: (i64,) =
            sqlx::query_as("select count(distinct mail_id) from mail_attachments")
                .fetch_one(&self.pool)
                .await?;
        let rows = sqlx::query_as::<_, MailAdminRow>(
            r#"
            select
              m.id,
              m.to_character_id,
              c.name as to_character_name,
              c.account_id,
              a.username as account_username,
              m.from_account_id,
              m.from_name,
              m.title,
              m.read,
              m.claimed,
              m.expires_at,
              m.created_at,
              count(ma.id) as attachment_count,
              coalesce(sum(ma.gold), 0)::bigint as attachment_gold,
              coalesce(sum(ma.yuanbao), 0)::bigint as attachment_yuanbao
            from mails m
            left join characters c on c.id = m.to_character_id
            left join accounts a on a.id = c.account_id
            left join mail_attachments ma on ma.mail_id = m.id
            group by m.id, c.name, c.account_id, a.username
            order by m.created_at desc
            limit 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminMailOverview {
            total: summary.total,
            unread: summary.unread,
            unclaimed: summary.unclaimed,
            with_attachments: with_attachments.0,
            recent: rows
                .into_iter()
                .map(|row| AdminMailView {
                    id: row.id,
                    to_character_id: row.to_character_id,
                    to_character_name: row.to_character_name,
                    account_id: row.account_id,
                    account_username: row.account_username,
                    from_account_id: row.from_account_id,
                    from_name: row.from_name,
                    title: row.title,
                    read: row.read,
                    claimed: row.claimed,
                    expires_at: row.expires_at.map(|value| value.to_string()),
                    created_at: row.created_at.to_string(),
                    attachment_count: row.attachment_count,
                    attachment_gold: row.attachment_gold,
                    attachment_yuanbao: row.attachment_yuanbao,
                })
                .collect(),
        })
    }

    pub async fn item_templates(&self) -> Result<AdminItemTemplateList, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("select count(*) from item_templates")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, ItemTemplateAdminRow>(
            r#"
            select id, name, kind, slot, rarity, price, stackable, stats, flags, version, created_at, updated_at
            from item_templates
            order by id asc
            limit 200
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminItemTemplateList {
            total: total.0,
            items: rows
                .into_iter()
                .map(|row| AdminItemTemplateView {
                    id: row.id,
                    name: row.name,
                    kind: row.kind,
                    slot: row.slot,
                    rarity: row.rarity,
                    price: row.price,
                    stackable: row.stackable,
                    stats: row.stats,
                    flags: row.flags,
                    version: row.version,
                    created_at: row.created_at.to_string(),
                    updated_at: row.updated_at.to_string(),
                })
                .collect(),
        })
    }

    pub async fn mob_templates(&self) -> Result<AdminMobTemplateList, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("select count(*) from mob_templates")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, MobTemplateAdminRow>(
            r#"
            select id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds, drops, version, created_at, updated_at
            from mob_templates
            order by level asc, boss desc, id asc
            limit 200
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminMobTemplateList {
            total: total.0,
            mobs: rows
                .into_iter()
                .map(|row| AdminMobTemplateView {
                    id: row.id,
                    name: row.name,
                    level: row.level,
                    max_hp: row.max_hp,
                    atk: row.atk,
                    def: row.def,
                    exp: row.exp,
                    gold: row.gold,
                    boss: row.boss,
                    respawn_seconds: row.respawn_seconds,
                    drops: row.drops,
                    version: row.version,
                    created_at: row.created_at.to_string(),
                    updated_at: row.updated_at.to_string(),
                })
                .collect(),
        })
    }

    pub async fn audit_logs(&self) -> Result<AdminAuditLogList, sqlx::Error> {
        let total: (i64,) = sqlx::query_as("select count(*) from admin_audit_logs")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, AuditLogAdminRow>(
            r#"
            select
              l.id,
              l.admin_account_id,
              a.username as admin_username,
              l.action,
              l.target,
              l.detail,
              l.ip::text as ip,
              l.created_at
            from admin_audit_logs l
            left join accounts a on a.id = l.admin_account_id
            order by l.created_at desc
            limit 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminAuditLogList {
            total: total.0,
            logs: rows
                .into_iter()
                .map(|row| AdminAuditLogView {
                    id: row.id,
                    admin_account_id: row.admin_account_id,
                    admin_username: row.admin_username,
                    action: row.action,
                    target: row.target,
                    detail: row.detail,
                    ip: row.ip,
                    created_at: row.created_at.to_string(),
                })
                .collect(),
        })
    }

    pub async fn audit(
        &self,
        admin_account_id: Option<i64>,
        action: &str,
        target: &str,
        detail: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into admin_audit_logs (admin_account_id, action, target, detail, ip)
            values ($1, $2, $3, $4, null)
            "#,
        )
        .bind(admin_account_id)
        .bind(action)
        .bind(target)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn send_system_mail(
        &self,
        to_character_id: Option<i64>,
        title: &str,
        body: &str,
        gold: i64,
        yuanbao: i64,
        item_template_id: Option<&str>,
        quantity: i64,
    ) -> Result<i64, sqlx::Error> {
        let target_count: (i64,) = sqlx::query_as(
            r#"
            select count(*)
            from characters
            where deleted_at is null
              and ($1::bigint is null or id = $1)
            "#,
        )
        .bind(to_character_id)
        .fetch_one(&self.pool)
        .await?;
        if target_count.0 == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        if let Some(template_id) = item_template_id {
            let exists: Option<(String,)> = sqlx::query_as("select id from item_templates where id = $1")
                .bind(template_id)
                .fetch_optional(&self.pool)
                .await?;
            if exists.is_none() {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        let mut tx = self.pool.begin().await?;
        let mail_ids = sqlx::query_as::<_, (i64,)>(
            r#"
            insert into mails (to_character_id, from_name, title, body)
            select id, 'GM', $2, $3
            from characters
            where deleted_at is null
              and ($1::bigint is null or id = $1)
            returning id
            "#,
        )
        .bind(to_character_id)
        .bind(title)
        .bind(body)
        .fetch_all(&mut *tx)
        .await?;

        if gold > 0 || yuanbao > 0 || item_template_id.is_some() {
            for (mail_id,) in &mail_ids {
                sqlx::query(
                    r#"
                    insert into mail_attachments (mail_id, item_template_id, quantity, gold, yuanbao)
                    values ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(*mail_id)
                .bind(item_template_id)
                .bind(quantity.max(1))
                .bind(gold.max(0))
                .bind(yuanbao.max(0))
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        Ok(mail_ids.len() as i64)
    }

    pub async fn update_character_detail(
        &self,
        character_id: i64,
        name: Option<&str>,
        class: Option<&str>,
        level: Option<i32>,
        exp: Option<i64>,
        gold: Option<i64>,
        yuanbao: Option<i64>,
        power: Option<i64>,
        zone: Option<&str>,
        room: Option<&str>,
        hp: Option<i64>,
        mp: Option<i64>,
        online: Option<bool>,
    ) -> Result<(), sqlx::Error> {
        let current = sqlx::query_as::<_, CharacterAssetRow>(
            r#"
            select class, level, exp
            from characters
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        let Some(current) = current else {
            return Err(sqlx::Error::RowNotFound);
        };

        let next_class = class.unwrap_or(&current.class).trim();
        let next_level = level.unwrap_or(current.level).clamp(1, 500);
        let next_exp = exp.unwrap_or(current.exp).max(0);
        let stats = initial_stats(class_from_str(next_class), next_level);
        let next_power = power.unwrap_or_else(|| power_from_stats(&stats)).max(0);

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            update characters
            set name = coalesce(nullif($2, ''), name),
                class = $3,
                level = $4,
                exp = $5,
                gold = coalesce($6, gold),
                yuanbao = coalesce($7, yuanbao),
                power = $8
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .bind(name.unwrap_or_default().trim())
        .bind(next_class)
        .bind(next_level)
        .bind(next_exp)
        .bind(gold.map(|value| value.max(0)))
        .bind(yuanbao.map(|value| value.max(0)))
        .bind(next_power)
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
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            update character_state
            set zone = coalesce(nullif($2, ''), zone),
                room = coalesce(nullif($3, ''), room),
                hp = coalesce($4, hp),
                mp = coalesce($5, mp),
                online = coalesce($6, online)
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(zone.unwrap_or_default().trim())
        .bind(room.unwrap_or_default().trim())
        .bind(hp)
        .bind(mp)
        .bind(online)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_character_item(
        &self,
        character_id: i64,
        item_id: Option<i64>,
        template_id: &str,
        quantity: i64,
        location: &str,
        slot: Option<&str>,
        bind: bool,
        durability: i32,
        extra: &Value,
    ) -> Result<(), sqlx::Error> {
        let character_exists: Option<(i64,)> =
            sqlx::query_as("select id from characters where id = $1 and deleted_at is null")
                .bind(character_id)
                .fetch_optional(&self.pool)
                .await?;
        if character_exists.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }
        let template_exists: Option<(String,)> = sqlx::query_as("select id from item_templates where id = $1")
            .bind(template_id)
            .fetch_optional(&self.pool)
            .await?;
        if template_exists.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Some(item_id) = item_id {
            let result = sqlx::query(
                r#"
                update inventory_items
                set template_id = $3,
                    quantity = $4,
                    location = $5,
                    slot = nullif($6, ''),
                    bind = $7,
                    durability = $8,
                    extra = $9
                where id = $1 and character_id = $2
                "#,
            )
            .bind(item_id)
            .bind(character_id)
            .bind(template_id)
            .bind(quantity.max(1))
            .bind(location)
            .bind(slot.unwrap_or_default().trim())
            .bind(bind)
            .bind(durability.clamp(0, 100))
            .bind(extra)
            .execute(&self.pool)
            .await?;
            if result.rows_affected() == 0 {
                return Err(sqlx::Error::RowNotFound);
            }
        } else {
            sqlx::query(
                r#"
                insert into inventory_items
                  (character_id, template_id, quantity, location, slot, bind, durability, extra)
                values ($1, $2, $3, $4, nullif($5, ''), $6, $7, $8)
                "#,
            )
            .bind(character_id)
            .bind(template_id)
            .bind(quantity.max(1))
            .bind(location)
            .bind(slot.unwrap_or_default().trim())
            .bind(bind)
            .bind(durability.clamp(0, 100))
            .bind(extra)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn delete_character_item(&self, character_id: i64, item_id: i64) -> Result<(), sqlx::Error> {
        let result = sqlx::query("delete from inventory_items where character_id = $1 and id = $2")
            .bind(character_id)
            .bind(item_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn adjust_character_assets(
        &self,
        character_id: i64,
        exp_delta: i64,
        gold_delta: i64,
        yuanbao_delta: i64,
    ) -> Result<(), sqlx::Error> {
        let current = sqlx::query_as::<_, CharacterAssetRow>(
            r#"
            select class, level, exp
            from characters
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;
        let Some(current) = current else {
            return Err(sqlx::Error::RowNotFound);
        };

        let next_exp = current.exp.saturating_add(exp_delta).max(0);
        let mut next_level = current.level;
        while next_level > 1 && next_exp < exp_for_level(next_level) {
            next_level -= 1;
        }
        while next_level < 500 && next_exp >= exp_for_level(next_level + 1) {
            next_level += 1;
        }
        let stats = initial_stats(class_from_str(&current.class), next_level);
        let power = power_from_stats(&stats);

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            update characters
            set exp = $2,
                gold = greatest(0, gold + $3),
                yuanbao = greatest(0, yuanbao + $4),
                level = $5,
                power = $6
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .bind(next_exp)
        .bind(gold_delta)
        .bind(yuanbao_delta)
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
        .execute(&mut *tx)
        .await?;
        if next_level != current.level {
            sqlx::query(
                r#"
                update character_state
                set hp = $2, mp = $3
                where character_id = $1
                "#,
            )
            .bind(character_id)
            .bind(stats.max_hp)
            .bind(stats.max_mp)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn set_activity_enabled(&self, code: &str, enabled: bool) -> Result<(), sqlx::Error> {
        let result = sqlx::query("update activities set enabled = $2 where code = $1")
            .bind(code)
            .bind(enabled)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn set_account_status(
        &self,
        account_id: i64,
        status: &str,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            update accounts
            set status = $2,
                banned_reason = case when $2 = 'banned' then nullif($3, '') else null end
            where id = $1
            "#,
        )
        .bind(account_id)
        .bind(status)
        .bind(reason.unwrap_or_default())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        if status == "banned" {
            sqlx::query("update sessions set revoked_at = coalesce(revoked_at, now()) where account_id = $1")
                .bind(account_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn set_character_state(
        &self,
        character_id: i64,
        zone: Option<&str>,
        room: Option<&str>,
        hp: Option<i64>,
        mp: Option<i64>,
        online: Option<bool>,
        force_offline: bool,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let next_online = if force_offline { Some(false) } else { online };
        let result = sqlx::query(
            r#"
            update character_state
            set zone = coalesce(nullif($2, ''), zone),
                room = coalesce(nullif($3, ''), room),
                hp = coalesce($4, hp),
                mp = coalesce($5, mp),
                online = coalesce($6, online),
                temp_state = case
                    when $7 then temp_state || jsonb_build_object(
                        'gm_kick', true,
                        'gm_kick_reason', $8,
                        'gm_kick_at', now()
                    )
                    else temp_state
                end
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(zone.unwrap_or_default())
        .bind(room.unwrap_or_default())
        .bind(hp)
        .bind(mp)
        .bind(next_online)
        .bind(force_offline)
        .bind(reason.unwrap_or_default())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn upsert_item_template(&self, item: &AdminItemTemplateInput) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags)
            values ($1, $2, $3, nullif($4, ''), $5, $6, $7, $8, $9)
            on conflict (id) do update
            set name = excluded.name,
                kind = excluded.kind,
                slot = excluded.slot,
                rarity = excluded.rarity,
                price = excluded.price,
                stackable = excluded.stackable,
                stats = excluded.stats,
                flags = excluded.flags,
                version = item_templates.version + 1
            "#,
        )
        .bind(item.id.trim())
        .bind(item.name.trim())
        .bind(item.kind.trim())
        .bind(item.slot.as_deref().unwrap_or_default().trim())
        .bind(item.rarity.trim())
        .bind(item.price.max(0))
        .bind(item.stackable)
        .bind(&item.stats)
        .bind(&item.flags)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct AccountAdminRow {
    id: i64,
    username: String,
    email: Option<String>,
    status: String,
    created_at: OffsetDateTime,
    last_login_at: Option<OffsetDateTime>,
    character_count: i64,
}

#[derive(Debug, FromRow)]
struct CharacterAdminRow {
    id: i64,
    account_id: i64,
    account_username: String,
    name: String,
    class: String,
    level: i32,
    exp: i64,
    gold: i64,
    yuanbao: i64,
    power: i64,
    zone: Option<String>,
    room: Option<String>,
    hp: Option<i64>,
    mp: Option<i64>,
    online: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, FromRow)]
struct MailSummaryRow {
    total: i64,
    unread: i64,
    unclaimed: i64,
}

#[derive(Debug, FromRow)]
struct MailAdminRow {
    id: i64,
    to_character_id: i64,
    to_character_name: Option<String>,
    account_id: Option<i64>,
    account_username: Option<String>,
    from_account_id: Option<i64>,
    from_name: String,
    title: String,
    read: bool,
    claimed: bool,
    expires_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    attachment_count: i64,
    attachment_gold: i64,
    attachment_yuanbao: i64,
}

#[derive(Debug, FromRow)]
struct ItemTemplateAdminRow {
    id: String,
    name: String,
    kind: String,
    slot: Option<String>,
    rarity: String,
    price: i64,
    stackable: bool,
    stats: Value,
    flags: Value,
    version: i64,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, FromRow)]
struct MobTemplateAdminRow {
    id: String,
    name: String,
    level: i32,
    max_hp: i64,
    atk: i64,
    def: i64,
    exp: i64,
    gold: i64,
    boss: bool,
    respawn_seconds: i32,
    drops: Value,
    version: i64,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, FromRow)]
struct AuditLogAdminRow {
    id: i64,
    admin_account_id: Option<i64>,
    admin_username: Option<String>,
    action: String,
    target: String,
    detail: Value,
    ip: Option<String>,
    created_at: OffsetDateTime,
}

#[derive(Debug, FromRow)]
struct CharacterAssetRow {
    class: String,
    level: i32,
    exp: i64,
}

fn character_admin_item(row: CharacterAdminRow) -> AdminCharacterListItem {
    AdminCharacterListItem {
        id: row.id,
        account_id: row.account_id,
        account_username: row.account_username,
        name: row.name,
        class: row.class,
        level: row.level,
        exp: row.exp,
        gold: row.gold,
        yuanbao: row.yuanbao,
        power: row.power,
        zone: row.zone,
        room: row.room,
        hp: row.hp,
        mp: row.mp,
        online: row.online,
        created_at: row.created_at.to_string(),
        updated_at: row.updated_at.to_string(),
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
