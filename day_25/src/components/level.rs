use bevy::prelude::*;

/// Marker component for entities that belong to the current level layout.
#[derive(Component, Default)]
pub struct LevelEntity;

/// Component identifying the portal that leads to another level.
#[derive(Component, Debug, Clone, Copy)]
pub struct LevelExitDoor {
    pub target_level: usize,
}

impl LevelExitDoor {
    pub fn new(target_level: usize) -> Self {
        Self { target_level }
    }
}
