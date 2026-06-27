use cq_domain::map::{starter_world, Position, Room, World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("room not found")]
    RoomNotFound,
    #[error("exit not found")]
    ExitNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldService {
    world: World,
}

impl Default for WorldService {
    fn default() -> Self {
        Self { world: starter_world() }
    }
}

impl WorldService {
    pub fn new(world: World) -> Self {
        Self { world }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn current_room(&self, position: &Position) -> Result<&Room, WorldError> {
        self.world.room(position).ok_or(WorldError::RoomNotFound)
    }

    pub fn move_to(&self, position: &Position, direction: &str) -> Result<(Position, Room), WorldError> {
        let next = self.world.resolve_exit(position, direction).ok_or(WorldError::ExitNotFound)?;
        let room = self.current_room(&next)?.clone();
        Ok((next, room))
    }
}
