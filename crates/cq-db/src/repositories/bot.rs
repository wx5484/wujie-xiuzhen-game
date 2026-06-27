use cq_domain::character::exp_for_level;
use cq_protocol::dto::{AdminBotBatchRequest, AdminBotCreateRequest, AdminBotList, AdminBotView};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone)]
pub struct BotRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, FromRow)]
struct BotRow {
    id: i64,
    name: String,
    bot_class: String,
    level: i32,
    exp: i64,
    gold: i64,
    power: i64,
    zone: String,
    room: String,
    hp: i64,
    mp: i64,
    mode: String,
    team_code: String,
    target_zone: String,
    target_room: String,
    enabled: bool,
    script: serde_json::Value,
    last_action_at: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct BotPvpTarget {
    pub source: String,
    pub id: i64,
    pub name: String,
    pub bot_class: String,
    pub level: i32,
    pub power: i64,
    pub hp: i64,
    pub mp: i64,
}

impl BotRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list(&self) -> Result<AdminBotList, sqlx::Error> {
        let (total,): (i64,) = sqlx::query_as("select count(*)::bigint from bot_profiles")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, BotRow>(
            r#"
            select id, name, bot_class, level, exp, gold, power,
                   zone, room, hp, mp, mode, team_code, target_zone, target_room,
                   enabled, script, last_action_at::text as last_action_at, updated_at::text as updated_at
            from bot_profiles
            order by enabled desc, level desc, id asc
            limit 200
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(AdminBotList { total, bots: rows.into_iter().map(bot_view).collect() })
    }

    pub async fn names_at(
        &self,
        zone: &str,
        room: &str,
        current_character_id: Option<i64>,
    ) -> Result<Vec<String>, sqlx::Error> {
        let player_rows = sqlx::query_as::<_, (String, i32, i64, i64)>(
            r#"
            select c.name, c.level, cs.hp, cs.mp
            from character_state cs
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.online = true
              and cs.zone = $1
              and cs.room = $2
              and ($3::bigint is null or c.id <> $3)
              and cs.updated_at > now() - interval '30 minutes'
            order by c.level desc, c.id asc
            limit 12
            "#,
        )
        .bind(zone)
        .bind(room)
        .bind(current_character_id)
        .fetch_all(&self.pool)
        .await?;
        let bot_rows = sqlx::query_as::<_, (String, i32, String, String, i64, i64)>(
            r#"
            select name, level, mode, team_code, hp, mp
            from bot_profiles
            where enabled = true and zone = $1 and room = $2
            order by level desc, id asc
            limit 12
            "#,
        )
        .bind(zone)
        .bind(room)
        .fetch_all(&self.pool)
        .await?;
        let mut names = player_rows
            .into_iter()
            .map(|(name, level, hp, mp)| {
                format!("{} · {}级 · 生命 {} · 魔法 {} · 在线玩家", name, level, hp, mp)
            })
            .collect::<Vec<_>>();
        names.extend(
            bot_rows
                .into_iter()
                .map(|(name, level, mode, team, hp, mp)| {
                    let team = if team.trim().is_empty() { "散人".into() } else { team };
                    format!("{} · {}级 · 生命 {} · 魔法 {} · {} · {}", name, level, hp, mp, mode_label(&mode), team)
                }),
        );
        Ok(names)
    }

    pub async fn pvp_target_at(
        &self,
        zone: &str,
        room: &str,
        attacker_character_id: i64,
        target_index: i64,
    ) -> Result<Option<BotPvpTarget>, sqlx::Error> {
        if target_index < 0 {
            return Ok(None);
        }
        let player_rows = sqlx::query_as::<_, BotPvpTarget>(
            r#"
            select 'player'::text as source,
                   c.id,
                   c.name,
                   c.class as bot_class,
                   c.level,
                   c.power,
                   cs.hp,
                   cs.mp
            from character_state cs
            join characters c on c.id = cs.character_id and c.deleted_at is null
            where cs.online = true
              and cs.zone = $1
              and cs.room = $2
              and c.id <> $3
              and cs.updated_at > now() - interval '30 minutes'
            order by c.level desc, c.id asc
            limit 12
            "#,
        )
        .bind(zone)
        .bind(room)
        .bind(attacker_character_id)
        .fetch_all(&self.pool)
        .await?;
        let bot_rows = sqlx::query_as::<_, BotPvpTarget>(
            r#"
            select 'bot'::text as source, id, name, bot_class, level, power, hp, mp
            from bot_profiles
            where enabled = true and zone = $1 and room = $2
            order by level desc, id asc
            limit 12
            "#,
        )
        .bind(zone)
        .bind(room)
        .fetch_all(&self.pool)
        .await?;
        let mut targets = player_rows;
        targets.extend(bot_rows);
        Ok(targets.get(target_index as usize).cloned())
    }

    pub async fn save_pvp_state(
        &self,
        target: &BotPvpTarget,
        hp: i64,
        mp: i64,
        defeated: bool,
    ) -> Result<(), sqlx::Error> {
        if target.source == "player" {
            if defeated {
                let (max_hp, max_mp) = bot_resources(target.level);
                sqlx::query(
                    r#"
                    update character_state
                    set zone = 'fanchen',
                        room = 'qingniu_city',
                        hp = $2,
                        mp = $3,
                        updated_at = now(),
                        last_idle_regen_at = now()
                    where character_id = $1
                    "#,
                )
                .bind(target.id)
                .bind((max_hp / 2).max(1))
                .bind((max_mp / 2).max(0))
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query(
                    r#"
                    update character_state
                    set hp = $2,
                        mp = $3,
                        updated_at = now(),
                        last_idle_regen_at = now()
                    where character_id = $1
                    "#,
                )
                .bind(target.id)
                .bind(hp.max(1))
                .bind(mp.max(0))
                .execute(&self.pool)
                .await?;
            }
            return Ok(());
        }

        if defeated {
            let (max_hp, max_mp) = bot_resources(target.level);
            sqlx::query(
                r#"
                update bot_profiles
                set zone = 'fanchen',
                    room = 'qingniu_city',
                    hp = $2,
                    mp = $3,
                    script = script || '{"last_pvp":"defeated"}'::jsonb,
                    last_action_at = now()
                where id = $1
                "#,
            )
            .bind(target.id)
            .bind((max_hp / 2).max(1))
            .bind((max_mp / 2).max(0))
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                update bot_profiles
                set hp = $2,
                    mp = $3,
                    script = script || '{"last_pvp":"attacked"}'::jsonb,
                    last_action_at = now()
                where id = $1
                "#,
            )
            .bind(target.id)
            .bind(hp.max(1))
            .bind(mp.max(0))
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn batch_config(&self, input: AdminBotBatchRequest) -> Result<AdminBotList, sqlx::Error> {
        let bot_ids = input.bot_ids.clone();
        let ids = if bot_ids.is_empty() {
            sqlx::query_as::<_, (i64,)>("select id from bot_profiles order by id asc")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|(id,)| id)
                .collect::<Vec<_>>()
        } else {
            bot_ids.into_iter().filter(|id| *id > 0).collect::<Vec<_>>()
        };

        for id in ids {
            sqlx::query(
                r#"
                update bot_profiles
                set mode = case when $2::text is null or $2 = '' then mode else $2 end,
                    enabled = coalesce($3, enabled),
                    zone = case when $4::text is null or $4 = '' then zone else $4 end,
                    room = case when $5::text is null or $5 = '' then room else $5 end,
                    team_code = case when $6::text is null then team_code else $6 end,
                    target_zone = case when $7::text is null then target_zone else $7 end,
                    target_room = case when $8::text is null then target_room else $8 end,
                    script = case when $9::jsonb = '{}'::jsonb then script else script || $9::jsonb end
                where id = $1
                "#,
            )
            .bind(id)
            .bind(input.mode.as_deref().map(str::trim).filter(|value| !value.is_empty()))
            .bind(input.enabled)
            .bind(input.zone.as_deref().map(str::trim).filter(|value| !value.is_empty()))
            .bind(input.room.as_deref().map(str::trim).filter(|value| !value.is_empty()))
            .bind(input.team_code.as_deref().map(str::trim))
            .bind(input.target_zone.as_deref().map(str::trim))
            .bind(input.target_room.as_deref().map(str::trim))
            .bind(if input.script.is_object() { input.script.clone() } else { serde_json::json!({}) })
            .execute(&self.pool)
            .await?;
        }

        self.list().await
    }

    pub async fn create(&self, input: AdminBotCreateRequest) -> Result<AdminBotList, sqlx::Error> {
        let name = input.name.trim();
        let bot_class = normalize_class(&input.bot_class);
        let name_len = name.chars().count();
        if !(2..=16).contains(&name_len) {
            return Err(sqlx::Error::RowNotFound);
        }
        let (hp, mp) = bot_resources(1);
        let power = bot_power(bot_class, 1);
        sqlx::query(
            r#"
            insert into bot_profiles
              (name, bot_class, level, exp, gold, power, zone, room, hp, mp, mode, team_code, target_zone, target_room, script)
            values
              ($1, $2, 1, 0, 0, $3, 'fanchen', 'bamboo_outer', $4, $5, 'progression', '', '', '', '{"note":"GM 新增 bot，从新手区自然成长"}')
            "#,
        )
        .bind(name)
        .bind(bot_class)
        .bind(power)
        .bind(hp)
        .bind(mp)
        .execute(&self.pool)
        .await?;
        self.list().await
    }

    pub async fn delete(&self, bot_id: i64) -> Result<AdminBotList, sqlx::Error> {
        sqlx::query("delete from bot_profiles where id = $1")
            .bind(bot_id)
            .execute(&self.pool)
            .await?;
        self.list().await
    }

    pub async fn tick(&self, limit: i64) -> Result<(AdminBotList, i64), sqlx::Error> {
        let rows = sqlx::query_as::<_, BotRow>(
            r#"
            select id, name, bot_class, level, exp, gold, power,
                   zone, room, hp, mp, mode, team_code, target_zone, target_room,
                   enabled, script, last_action_at::text as last_action_at, updated_at::text as updated_at
            from bot_profiles
            where enabled = true
            order by last_action_at asc nulls first, id asc
            limit $1
            "#,
        )
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await?;
        let mut changed = 0_i64;
        for row in rows {
            let (zone, room) = next_position(&row);
            let team_bonus_pct = self.team_bonus_pct(&row, &zone, &room).await?;
            let exp_gain = bot_exp_gain(&row, team_bonus_pct);
            let gold_gain = bot_gold_gain(&row, team_bonus_pct);
            let mut next_level = row.level;
            let next_exp = row.exp.saturating_add(exp_gain);
            while next_level < 500 && next_exp >= exp_for_level(next_level + 1) {
                next_level += 1;
            }
            let (hp, mp) = bot_resources(next_level);
            let power = bot_power(&row.bot_class, next_level);
            let script_patch = serde_json::json!({
                "last_gain": { "exp": exp_gain, "gold": gold_gain, "team_bonus_pct": team_bonus_pct },
                "last_mode": row.mode,
            });
            sqlx::query(
                r#"
                update bot_profiles
                set level = $2,
                    exp = $3,
                    gold = gold + $4,
                    power = $5,
                    zone = $6,
                    room = $7,
                    hp = $8,
                    mp = $9,
                    script = script || $10::jsonb,
                    last_action_at = now()
                where id = $1
                "#,
            )
            .bind(row.id)
            .bind(next_level)
            .bind(next_exp)
            .bind(gold_gain)
            .bind(power)
            .bind(zone)
            .bind(room)
            .bind(hp)
            .bind(mp)
            .bind(script_patch)
            .execute(&self.pool)
            .await?;
            changed += 1;
        }

        Ok((self.list().await?, changed))
    }

    async fn team_bonus_pct(&self, row: &BotRow, zone: &str, room: &str) -> Result<i64, sqlx::Error> {
        if row.team_code.trim().is_empty() {
            return Ok(0);
        }
        let (count,): (i64,) = sqlx::query_as(
            r#"
            select count(*)::bigint
            from bot_profiles
            where enabled = true
              and team_code = $1
              and zone = $2
              and room = $3
            "#,
        )
        .bind(row.team_code.trim())
        .bind(zone)
        .bind(room)
        .fetch_one(&self.pool)
        .await?;
        Ok(((count.saturating_sub(1)) * 10).clamp(0, 30))
    }
}

fn bot_view(row: BotRow) -> AdminBotView {
    AdminBotView {
        id: row.id,
        name: row.name,
        bot_class: row.bot_class,
        level: row.level,
        exp: row.exp,
        gold: row.gold,
        power: row.power,
        zone: row.zone,
        room: row.room,
        hp: row.hp,
        mp: row.mp,
        mode: row.mode,
        team_code: row.team_code,
        target_zone: row.target_zone,
        target_room: row.target_room,
        enabled: row.enabled,
        script: row.script,
        last_action_at: row.last_action_at,
        updated_at: row.updated_at,
    }
}

fn next_position(row: &BotRow) -> (String, String) {
    let target = (
        non_empty(&row.target_zone).unwrap_or(&row.zone).to_owned(),
        non_empty(&row.target_room).unwrap_or(&row.room).to_owned(),
    );
    match row.mode.as_str() {
        "dispatch" | "team_farm" | "fixed_clear" => target,
        _ => progression_position(row.level),
    }
}

fn progression_position(level: i32) -> (String, String) {
    match level {
        1..=3 => ("fanchen".into(), "bamboo_outer".into()),
        4..=6 => ("fanchen".into(), "bamboo_fork".into()),
        7..=10 => ("fanchen".into(), "dense_river".into()),
        11..=17 => ("fanchen".into(), "mine_entrance".into()),
        18..=20 => ("fanchen".into(), "collapsed_passage".into()),
        21..=28 => ("fanchen".into(), "wilderness_edge".into()),
        29..=35 => ("fanchen".into(), "bone_highland".into()),
        36..=40 => ("fanchen".into(), "wilderness_depths".into()),
        41..=55 => ("xiuzhen".into(), "poison_marsh".into()),
        56..=68 => ("xiuzhen".into(), "flower_secret".into()),
        69..=80 => ("xiuzhen".into(), "ape_ravine".into()),
        81..=95 => ("xiuzhen".into(), "tower_floor_1".into()),
        96..=110 => ("xiuzhen".into(), "tower_floor_2".into()),
        111..=120 => ("xiuzhen".into(), "tower_floor_3".into()),
        121..=135 => ("xiuzhen".into(), "ice_forest".into()),
        136..=150 => ("xiuzhen".into(), "winter_canyon".into()),
        151..=160 => ("xiuzhen".into(), "dragonbone_icefield".into()),
        161..=185 => ("feisheng".into(), "wangchuan_bank".into()),
        186..=200 => ("feisheng".into(), "nether_water_palace".into()),
        201..=230 => ("feisheng".into(), "moon_maze".into()),
        231..=240 => ("feisheng".into(), "moon_worship_platform".into()),
        241..=260 => ("feisheng".into(), "abyss_shallow".into()),
        261..=280 => ("feisheng".into(), "abyss_cave".into()),
        281..=300 => ("feisheng".into(), "chaos_storm".into()),
        301..=340 => ("ancient_secret".into(), "mining_outer".into()),
        341..=380 => ("ancient_secret".into(), "miasma_forest".into()),
        381..=420 => ("ancient_secret".into(), "ruined_gate".into()),
        421..=460 => ("ancient_secret".into(), "ash_plain".into()),
        461..=498 => ("ancient_secret".into(), "broken_stars".into()),
        _ => ("ancient_secret".into(), "heavenly_dao_altar".into()),
    }
}

fn bot_exp_gain(row: &BotRow, team_bonus_pct: i64) -> i64 {
    let mode_mul = match row.mode.as_str() {
        "fixed_clear" => 150,
        "team_farm" => 130,
        "dispatch" => 80,
        _ => 100,
    };
    let base = i64::from(row.level).saturating_mul(45).saturating_add(120);
    base.saturating_mul(mode_mul).saturating_mul(100 + team_bonus_pct) / 10_000
}

fn bot_gold_gain(row: &BotRow, team_bonus_pct: i64) -> i64 {
    let mode_mul = match row.mode.as_str() {
        "fixed_clear" => 120,
        "team_farm" => 110,
        "dispatch" => 50,
        _ => 80,
    };
    let base = i64::from(row.level).saturating_mul(9).saturating_add(20);
    base.saturating_mul(mode_mul).saturating_mul(100 + team_bonus_pct) / 10_000
}

fn bot_resources(level: i32) -> (i64, i64) {
    let level_bonus = i64::from(level.saturating_sub(1));
    (160 + level_bonus * 24, 60 + level_bonus * 8)
}

fn bot_power(class: &str, level: i32) -> i64 {
    let class_bonus = match class {
        "mage" => 18,
        "taoist" => 14,
        "assassin" => 20,
        _ => 16,
    };
    i64::from(level).saturating_mul(42 + class_bonus)
}

fn normalize_class(value: &str) -> &'static str {
    match value {
        "mage" => "mage",
        "taoist" => "taoist",
        "assassin" => "assassin",
        _ => "warrior",
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn mode_label(mode: &str) -> &'static str {
    match mode {
        "dispatch" => "调度",
        "team_farm" => "组队刷怪",
        "fixed_clear" => "清场",
        _ => "成长",
    }
}
