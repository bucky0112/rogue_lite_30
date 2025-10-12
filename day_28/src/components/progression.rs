use bevy::prelude::*;

use crate::constants::{
    PLAYER_LEVEL_BASE_ATTACK, PLAYER_LEVEL_BASE_DEFENSE, PLAYER_LEVEL_SPRITE_PATHS,
    PLAYER_LEVEL_XP_REQUIREMENTS, PLAYER_MAX_LEVEL,
};

#[derive(Component, Debug, Clone)]
pub struct PlayerProgression {
    pub level: usize,
    pub experience: u32,
}

impl PlayerProgression {
    pub fn new() -> Self {
        Self {
            level: 0,
            experience: 0,
        }
    }

    pub fn max_level() -> usize {
        PLAYER_MAX_LEVEL
    }

    pub fn next_level_requirement(&self) -> Option<u32> {
        PLAYER_LEVEL_XP_REQUIREMENTS.get(self.level).copied()
    }

    pub fn base_attack(&self) -> i32 {
        PLAYER_LEVEL_BASE_ATTACK
            .get(self.level)
            .copied()
            .unwrap_or_else(|| *PLAYER_LEVEL_BASE_ATTACK.last().unwrap())
    }

    pub fn base_defense(&self) -> i32 {
        PLAYER_LEVEL_BASE_DEFENSE
            .get(self.level)
            .copied()
            .unwrap_or_else(|| *PLAYER_LEVEL_BASE_DEFENSE.last().unwrap())
    }

    pub fn sprite_path(&self) -> &'static str {
        PLAYER_LEVEL_SPRITE_PATHS
            .get(self.level)
            .copied()
            .unwrap_or_else(|| *PLAYER_LEVEL_SPRITE_PATHS.last().unwrap())
    }
}
