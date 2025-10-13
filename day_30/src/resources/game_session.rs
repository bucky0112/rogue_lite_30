use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Paused,
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::MainMenu
    }
}

#[derive(Resource, Debug, Default)]
pub struct GameSession {
    phase: GamePhase,
    pub main_menu_root: Option<Entity>,
    pub pause_menu_root: Option<Entity>,
}

impl GameSession {
    pub const SAVE_DIRECTORY: &'static str = "saves";
    pub const SAVE_SLOT_FILE: &'static str = "saves/slot1.json";

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.phase, GamePhase::Playing)
    }
}
