use cq_domain::{
    character::{CharacterClass, CharacterStats, exp_for_level, initial_stats, power_from_stats},
    map::Position,
};
use cq_protocol::dto::{
    PlayerAttributesView, PlayerCharacterStateView, PlayerCharacterStatsView, PlayerCharacterView,
    PlayerRankingEntry,
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

pub const DAILY_STAMINA_LIMIT: i32 = 5_000;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CharacterRecord {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub class: String,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub power: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CharacterStateRecord {
    pub character_id: i64,
    pub zone: String,
    pub room: String,
    pub hp: i64,
    pub mp: i64,
    pub stamina: i32,
    pub stamina_max: i32,
    pub online: bool,
    pub pk_enabled: bool,
    pub sweep_attack_players: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CharacterStatsRecord {
    pub character_id: i64,
    pub str_: i64,
    pub dex: i64,
    pub int_: i64,
    pub con: i64,
    pub spirit: i64,
    pub max_hp: i64,
    pub max_mp: i64,
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
}

#[derive(Debug, Clone)]
pub struct IdleRecovery {
    pub state: CharacterStateRecord,
    pub minutes: i64,
    pub hp: i64,
    pub mp: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct StaminaConsumption {
    pub before_stamina: i32,
    pub stamina: i32,
    pub stamina_max: i32,
    pub full_kills: i64,
    pub fatigue_kills: i64,
}

#[derive(Debug, Clone, FromRow)]
struct RankingRow {
    rank: i64,
    character_id: i64,
    name: String,
    class: String,
    level: i32,
    exp: i64,
    gold: i64,
    yuanbao: i64,
    power: i64,
}

#[derive(Debug, Clone, FromRow)]
struct IdleRecoveryRow {
    character_id: i64,
    zone: String,
    room: String,
    hp: i64,
    mp: i64,
    stamina: i32,
    stamina_max: i32,
    online: bool,
    pk_enabled: bool,
    sweep_attack_players: bool,
    updated_at: String,
    idle_minutes: i64,
}

#[derive(Debug, Clone)]
pub struct CharacterRepository {
    pool: PgPool,
}

impl CharacterRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_for_account(&self, account_id: i64) -> Result<Vec<CharacterRecord>, sqlx::Error> {
        sqlx::query_as::<_, CharacterRecord>(
            r#"
            select id, account_id, name, class, level, exp, gold, yuanbao, power
            from characters
            where account_id = $1 and deleted_at is null
            order by id asc
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        account_id: i64,
        name: &str,
        class: CharacterClass,
    ) -> Result<CharacterRecord, sqlx::Error> {
        let stats = initial_stats(class, 1);
        let power = power_from_stats(&stats);
        let class_name = serde_json::to_value(class)
            .ok()
            .and_then(|v| v.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "warrior".into());
        let mut tx = self.pool.begin().await?;
        let character = sqlx::query_as::<_, CharacterRecord>(
            r#"
            insert into characters (account_id, name, class, level, exp, gold, yuanbao, power)
            values ($1, $2, $3, 1, 0, 100, 0, $4)
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(account_id)
        .bind(name)
        .bind(&class_name)
        .bind(power)
        .fetch_one(&mut *tx)
        .await?;
        insert_stats(&mut tx, character.id, &stats).await?;
        sqlx::query(
            r#"
            insert into character_state (character_id, zone, room, hp, mp)
            values ($1, 'fanchen', 'qingniu_city', $2, $3)
            "#,
        )
        .bind(character.id)
        .bind(stats.max_hp)
        .bind(stats.max_mp)
        .execute(&mut *tx)
        .await?;
        insert_welcome_mail(&mut tx, character.id).await?;
        tx.commit().await?;
        Ok(character)
    }

    pub async fn find(&self, character_id: i64) -> Result<Option<CharacterRecord>, sqlx::Error> {
        sqlx::query_as::<_, CharacterRecord>(
            r#"
            select id, account_id, name, class, level, exp, gold, yuanbao, power
            from characters
            where id = $1 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_for_account(
        &self,
        account_id: i64,
        character_id: i64,
    ) -> Result<Option<CharacterRecord>, sqlx::Error> {
        sqlx::query_as::<_, CharacterRecord>(
            r#"
            select id, account_id, name, class, level, exp, gold, yuanbao, power
            from characters
            where id = $1 and account_id = $2 and deleted_at is null
            "#,
        )
        .bind(character_id)
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn first_for_account(&self, account_id: i64) -> Result<Option<CharacterRecord>, sqlx::Error> {
        sqlx::query_as::<_, CharacterRecord>(
            r#"
            select id, account_id, name, class, level, exp, gold, yuanbao, power
            from characters
            where account_id = $1 and deleted_at is null
            order by id asc
            limit 1
            "#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn top_by_power(&self, limit: i64) -> Result<Vec<PlayerRankingEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RankingRow>(
            r#"
            select row_number() over (order by power desc, level desc, exp desc, id asc)::bigint as rank,
                   id as character_id,
                   name,
                   class,
                   level,
                   exp,
                   gold,
                   yuanbao,
                   power
            from characters
            where deleted_at is null
            order by power desc, level desc, exp desc, id asc
            limit $1
            "#,
        )
        .bind(limit.clamp(1, 50))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(ranking_entry).collect())
    }

    pub async fn top_by_level(&self, limit: i64) -> Result<Vec<PlayerRankingEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RankingRow>(
            r#"
            select row_number() over (order by level desc, exp desc, power desc, id asc)::bigint as rank,
                   id as character_id,
                   name,
                   class,
                   level,
                   exp,
                   gold,
                   yuanbao,
                   power
            from characters
            where deleted_at is null
            order by level desc, exp desc, power desc, id asc
            limit $1
            "#,
        )
        .bind(limit.clamp(1, 50))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(ranking_entry).collect())
    }

    pub async fn stats(&self, character_id: i64) -> Result<CharacterStatsRecord, sqlx::Error> {
        sqlx::query_as::<_, CharacterStatsRecord>(
            r#"
            select character_id,
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
        .await
    }

    pub async fn state(&self, character_id: i64) -> Result<CharacterStateRecord, sqlx::Error> {
        self.refresh_daily_stamina(character_id).await?;
        sqlx::query_as::<_, CharacterStateRecord>(
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
        .await
    }

    pub async fn refresh_daily_stamina(&self, character_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            update character_state
            set stamina = $2,
                stamina_max = $2,
                stamina_recovered_on = (now() at time zone 'Asia/Shanghai')::date
            where character_id = $1
              and stamina_recovered_on < (now() at time zone 'Asia/Shanghai')::date
            "#,
        )
        .bind(character_id)
        .bind(DAILY_STAMINA_LIMIT)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn consume_stamina_for_kills(
        &self,
        character_id: i64,
        kills: i64,
    ) -> Result<StaminaConsumption, sqlx::Error> {
        self.refresh_daily_stamina(character_id).await?;
        let kills = kills.max(0);
        if kills == 0 {
            let (stamina, stamina_max): (i32, i32) =
                sqlx::query_as("select stamina, stamina_max from character_state where character_id = $1")
                    .bind(character_id)
                    .fetch_one(&self.pool)
                    .await?;
            return Ok(StaminaConsumption {
                before_stamina: stamina,
                stamina,
                stamina_max,
                full_kills: 0,
                fatigue_kills: 0,
            });
        }

        sqlx::query_as::<_, StaminaConsumption>(
            r#"
            with current_state as (
              select stamina, stamina_max
              from character_state
              where character_id = $1
              for update
            ),
            next_state as (
              select
                stamina as before_stamina,
                greatest(stamina::bigint - $2::bigint, 0)::int as stamina,
                stamina_max,
                least(stamina::bigint, $2::bigint) as full_kills,
                greatest($2::bigint - stamina::bigint, 0) as fatigue_kills
              from current_state
            )
            update character_state cs
            set stamina = next_state.stamina,
                stamina_max = $3
            from next_state
            where cs.character_id = $1
            returning
              next_state.before_stamina,
              next_state.stamina,
              $3::int as stamina_max,
              next_state.full_kills,
              next_state.fatigue_kills
            "#,
        )
        .bind(character_id)
        .bind(kills)
        .bind(DAILY_STAMINA_LIMIT)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn consume_stamina_for_kill(&self, character_id: i64) -> Result<StaminaConsumption, sqlx::Error> {
        self.consume_stamina_for_kills(character_id, 1).await
    }

    pub async fn set_online(&self, character_id: i64, online: bool) -> Result<(), sqlx::Error> {
        sqlx::query("update character_state set online = $2, updated_at = now() where character_id = $1")
            .bind(character_id)
            .bind(online)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_position(&self, character_id: i64, position: &Position) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            update character_state
            set zone = $2,
                room = $3,
                last_idle_regen_at = now()
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(&position.zone)
        .bind(&position.room)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn spend_gold_and_save_position(
        &self,
        character_id: i64,
        gold_cost: i64,
        position: &Position,
    ) -> Result<CharacterRecord, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let character = sqlx::query_as::<_, CharacterRecord>(
            r#"
            update characters
            set gold = gold - $2
            where id = $1
              and deleted_at is null
              and gold >= $2
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(character_id)
        .bind(gold_cost.max(0))
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        sqlx::query(
            r#"
            update character_state
            set zone = $2,
                room = $3,
                last_idle_regen_at = now()
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .bind(&position.zone)
        .bind(&position.room)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(character)
    }

    pub async fn save_resources(
        &self,
        character_id: i64,
        hp: i64,
        mp: i64,
    ) -> Result<CharacterStateRecord, sqlx::Error> {
        sqlx::query_as::<_, CharacterStateRecord>(
            r#"
            update character_state
            set hp = greatest(0, $2),
                mp = greatest(0, $3),
                last_idle_regen_at = now()
            where character_id = $1
            returning
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
            "#,
        )
        .bind(character_id)
        .bind(hp)
        .bind(mp)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn save_state_snapshot(
        &self,
        character_id: i64,
        position: &Position,
        hp: i64,
        mp: i64,
    ) -> Result<CharacterStateRecord, sqlx::Error> {
        sqlx::query_as::<_, CharacterStateRecord>(
            r#"
            update character_state
            set zone = $2,
                room = $3,
                hp = greatest(0, $4),
                mp = greatest(0, $5),
                last_idle_regen_at = now()
            where character_id = $1
            returning
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
            "#,
        )
        .bind(character_id)
        .bind(&position.zone)
        .bind(&position.room)
        .bind(hp)
        .bind(mp)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_pk_settings(
        &self,
        character_id: i64,
        pk_enabled: bool,
        sweep_attack_players: bool,
    ) -> Result<CharacterStateRecord, sqlx::Error> {
        sqlx::query_as::<_, CharacterStateRecord>(
            r#"
            update character_state
            set pk_enabled = $2,
                sweep_attack_players = $3,
                last_idle_regen_at = now()
            where character_id = $1
            returning
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
            "#,
        )
        .bind(character_id)
        .bind(pk_enabled)
        .bind(sweep_attack_players)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn recover_idle_resources(
        &self,
        character_id: i64,
        max_hp: i64,
        max_mp: i64,
        hp_per_minute: i64,
        mp_per_minute: i64,
    ) -> Result<IdleRecovery, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, IdleRecoveryRow>(
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
              updated_at::text as updated_at,
              least(
                greatest(floor(extract(epoch from (now() - last_idle_regen_at)) / 60)::bigint, 0),
                60
              ) as idle_minutes
            from character_state
            where character_id = $1
            for update
            "#,
        )
        .bind(character_id)
        .fetch_one(&mut *tx)
        .await?;

        if row.idle_minutes <= 0 {
            tx.commit().await?;
            return Ok(IdleRecovery {
                state: idle_recovery_state(row),
                minutes: 0,
                hp: 0,
                mp: 0,
            });
        }

        let max_hp = max_hp.max(1);
        let max_mp = max_mp.max(0);
        let recovered_hp = row
            .idle_minutes
            .saturating_mul(hp_per_minute.max(0))
            .min(max_hp)
            .max(0);
        let recovered_mp = row
            .idle_minutes
            .saturating_mul(mp_per_minute.max(0))
            .min(max_mp)
            .max(0);
        let next_hp = (row.hp + recovered_hp).clamp(0, max_hp);
        let next_mp = (row.mp + recovered_mp).clamp(0, max_mp);
        let state = sqlx::query_as::<_, CharacterStateRecord>(
            r#"
            update character_state
            set hp = $2,
                mp = $3,
                last_idle_regen_at = last_idle_regen_at + ($4::bigint * interval '1 minute')
            where character_id = $1
            returning
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
            "#,
        )
        .bind(character_id)
        .bind(next_hp)
        .bind(next_mp)
        .bind(row.idle_minutes)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(IdleRecovery {
            state,
            minutes: row.idle_minutes,
            hp: next_hp.saturating_sub(row.hp),
            mp: next_mp.saturating_sub(row.mp),
        })
    }

    pub async fn grant_reward(
        &self,
        character_id: i64,
        exp: i64,
        gold: i64,
    ) -> Result<CharacterRecord, sqlx::Error> {
        let current = self
            .find(character_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        let mut next_level = current.level;
        let next_exp = current.exp.saturating_add(exp.max(0));
        while next_level < 500 && next_exp >= exp_for_level(next_level + 1) {
            next_level += 1;
        }
        let leveled = next_level != current.level;
        let next_stats = leveled.then(|| initial_stats(class_from_str(&current.class), next_level));
        let next_power = next_stats
            .as_ref()
            .map(power_from_stats)
            .unwrap_or(current.power);

        let mut tx = self.pool.begin().await?;
        let updated = sqlx::query_as::<_, CharacterRecord>(
            r#"
            update characters
            set exp = $2,
                gold = gold + $3,
                level = $4,
                power = $5
            where id = $1 and deleted_at is null
            returning id, account_id, name, class, level, exp, gold, yuanbao, power
            "#,
        )
        .bind(character_id)
        .bind(next_exp)
        .bind(gold.max(0))
        .bind(next_level)
        .bind(next_power)
        .fetch_one(&mut *tx)
        .await?;
        if let Some(stats) = next_stats {
            update_stats(&mut tx, character_id, &stats).await?;
            sqlx::query(
                r#"
                update character_state
                set hp = $2,
                    mp = $3,
                    last_idle_regen_at = now()
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
        Ok(updated)
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

pub fn character_view(record: CharacterRecord) -> PlayerCharacterView {
    PlayerCharacterView {
        id: record.id,
        account_id: record.account_id,
        name: record.name,
        class: record.class,
        level: record.level,
        exp: record.exp,
        gold: record.gold,
        yuanbao: record.yuanbao,
        power: record.power,
    }
}

pub fn stats_view(record: CharacterStatsRecord) -> PlayerCharacterStatsView {
    PlayerCharacterStatsView {
        character_id: record.character_id,
        attrs: PlayerAttributesView {
            str_: record.str_,
            dex: record.dex,
            int_: record.int_,
            con: record.con,
            spirit: record.spirit,
        },
        max_hp: record.max_hp,
        max_mp: record.max_mp,
        atk: record.atk,
        def: record.def,
        mag: record.mag,
        mdef: record.mdef,
    }
}

pub fn state_view(record: CharacterStateRecord) -> PlayerCharacterStateView {
    PlayerCharacterStateView {
        character_id: record.character_id,
        zone: record.zone,
        room: record.room,
        hp: record.hp,
        mp: record.mp,
        stamina: record.stamina,
        stamina_max: record.stamina_max,
        online: record.online,
        pk_enabled: record.pk_enabled,
        sweep_attack_players: record.sweep_attack_players,
        updated_at: record.updated_at,
    }
}

fn idle_recovery_state(row: IdleRecoveryRow) -> CharacterStateRecord {
    CharacterStateRecord {
        character_id: row.character_id,
        zone: row.zone,
        room: row.room,
        hp: row.hp,
        mp: row.mp,
        stamina: row.stamina,
        stamina_max: row.stamina_max,
        online: row.online,
        pk_enabled: row.pk_enabled,
        sweep_attack_players: row.sweep_attack_players,
        updated_at: row.updated_at,
    }
}

fn ranking_entry(row: RankingRow) -> PlayerRankingEntry {
    PlayerRankingEntry {
        rank: row.rank,
        character_id: row.character_id,
        name: row.name,
        class: row.class,
        level: row.level,
        exp: row.exp,
        gold: row.gold,
        yuanbao: row.yuanbao,
        power: row.power,
    }
}

async fn insert_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    character_id: i64,
    stats: &CharacterStats,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        insert into character_stats
          (character_id, "str", dex, "int", con, spirit, max_hp, max_mp, atk, def, mag, mdef)
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
    Ok(())
}

async fn insert_welcome_mail(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    character_id: i64,
) -> Result<(), sqlx::Error> {
    let (mail_id,): (i64,) = sqlx::query_as(
        r#"
        insert into mails (to_character_id, from_name, title, body)
        values (
          $1,
          '系统',
          '新手行囊',
          '欢迎来到比奇。这里有几件基础补给：药水、回城卷、木剑和启动金币。领取后可以直接出城练级。'
        )
        returning id
        "#,
    )
    .bind(character_id)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        insert into mail_attachments (mail_id, item_template_id, quantity, gold, yuanbao) values
          ($1, 'potion_small', 5, 0, 0),
          ($1, 'potion_mana', 3, 0, 0),
          ($1, 'scroll_return', 2, 0, 0),
          ($1, 'sword_wood', 1, 0, 0),
          ($1, null, 1, 500, 0)
        "#,
    )
    .bind(mail_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn update_stats(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    character_id: i64,
    stats: &CharacterStats,
) -> Result<(), sqlx::Error> {
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
    Ok(())
}
