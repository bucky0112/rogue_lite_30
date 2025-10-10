use crate::components::{ShieldKind, WeaponKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSaveData {
    pub version: u32,
    pub level_index: usize,
    pub player_health: i32,
    pub player_max_health: i32,
    pub player_level: usize,
    pub player_experience: u32,
    pub equipped_weapon: Option<WeaponKind>,
    pub equipped_shield: Option<ShieldKind>,
}

impl GameSaveData {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            level_index: 0,
            player_health: 0,
            player_max_health: 0,
            player_level: 0,
            player_experience: 0,
            equipped_weapon: None,
            equipped_shield: None,
        }
    }
}
