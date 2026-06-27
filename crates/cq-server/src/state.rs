use std::sync::Arc;

use cq_db::{repositories::world::WorldRepository, Db};
use cq_domain::{map::starter_world, mob::{starter_mobs, MobTemplate}};
use cq_game::world::WorldService;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Db,
    pub world: Arc<WorldService>,
    pub mobs: Arc<Vec<MobTemplate>>,
}

impl AppState {
    pub async fn new(config: Config, db: Db) -> Self {
        let world_repo = WorldRepository::new(db.pool());
        let world = match world_repo.world().await {
            Ok(world) if !world.zones.is_empty() => world,
            Ok(_) => starter_world(),
            Err(err) => {
                tracing::warn!(error = ?err, "failed to load world from database, using embedded starter world");
                starter_world()
            }
        };
        let mobs = match world_repo.domain_mob_templates().await {
            Ok(mobs) if !mobs.is_empty() => mobs,
            Ok(_) => starter_mobs(),
            Err(err) => {
                tracing::warn!(error = ?err, "failed to load mobs from database, using embedded starter mobs");
                starter_mobs()
            }
        };
        Self {
            config: Arc::new(config),
            db,
            world: Arc::new(WorldService::new(world)),
            mobs: Arc::new(mobs),
        }
    }
}
