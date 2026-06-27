use std::collections::BTreeMap;

use cq_domain::{map::{Room, World, Zone}, mob::MobTemplate};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MobTemplateRecord {
    pub id: String,
    pub name: String,
    pub level: i32,
    pub max_hp: i64,
    pub atk: i64,
    pub def: i64,
    pub exp: i64,
    pub gold: i64,
    pub boss: bool,
    pub respawn_seconds: i32,
}

#[derive(Debug, Clone, FromRow)]
struct ZoneRow {
    id: String,
    name: String,
}

#[derive(Debug, Clone, FromRow)]
struct RoomRow {
    zone_id: String,
    id: String,
    name: String,
    description: String,
    exits: serde_json::Value,
    spawns: serde_json::Value,
    safe: bool,
}

#[derive(Debug, Clone)]
pub struct WorldRepository {
    pool: PgPool,
}

impl WorldRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn mob_templates(&self) -> Result<Vec<MobTemplateRecord>, sqlx::Error> {
        sqlx::query_as::<_, MobTemplateRecord>(
            "select id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds from mob_templates order by level asc",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn domain_mob_templates(&self) -> Result<Vec<MobTemplate>, sqlx::Error> {
        let rows = self.mob_templates().await?;
        Ok(rows
            .into_iter()
            .map(|row| MobTemplate {
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
            })
            .collect())
    }

    pub async fn world(&self) -> Result<World, sqlx::Error> {
        let zones = sqlx::query_as::<_, ZoneRow>("select id, name from world_zones order by id asc")
            .fetch_all(&self.pool)
            .await?;
        let rooms = sqlx::query_as::<_, RoomRow>(
            r#"
            select zone_id, id, name, description, exits, spawns, safe
            from world_rooms
            order by zone_id asc, id asc
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut zone_map = BTreeMap::new();
        for zone in zones {
            zone_map.insert(
                zone.id.clone(),
                Zone {
                    id: zone.id,
                    name: zone.name,
                    rooms: BTreeMap::new(),
                },
            );
        }

        for row in rooms {
            if let Some(zone) = zone_map.get_mut(&row.zone_id) {
                let exits = match serde_json::from_value::<BTreeMap<String, String>>(row.exits) {
                    Ok(value) => value,
                    Err(_) => BTreeMap::new(),
                };
                let spawns = match serde_json::from_value::<Vec<String>>(row.spawns) {
                    Ok(value) => value,
                    Err(_) => Vec::new(),
                };
                zone.rooms.insert(
                    row.id.clone(),
                    Room {
                        id: row.id,
                        name: row.name,
                        desc: row.description,
                        exits,
                        spawns,
                        safe: row.safe,
                    },
                );
            }
        }

        Ok(World { zones: zone_map })
    }
}
