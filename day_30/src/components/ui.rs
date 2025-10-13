use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerHealthUiRoot;

#[derive(Component)]
pub struct PlayerHealthUiFill;

#[derive(Component)]
pub struct PlayerStaminaUiRoot;

#[derive(Component)]
pub struct PlayerStaminaUiFill;

#[derive(Component)]
pub struct PlayerStatusText;

#[derive(Component)]
pub struct DeathScreenRoot;

#[derive(Component)]
pub struct DeathScreenText;

#[derive(Component)]
pub struct HealthBarTarget {
    pub target: Entity,
}

impl HealthBarTarget {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

#[derive(Component)]
pub struct HealthBarFollow {
    pub target: Entity,
    pub offset: Vec3,
}

impl HealthBarFollow {
    pub fn new(target: Entity, offset: Vec3) -> Self {
        Self { target, offset }
    }
}

#[derive(Component)]
pub struct EnemyHealthBarRoot;

#[derive(Component)]
pub struct EnemyHealthBarFill;

#[derive(Component)]
pub struct PlayerStatsPanel;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct MainMenuButton {
    pub action: MainMenuAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainMenuAction {
    NewGame,
    LoadGame,
}

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct PauseMenuButton {
    pub action: PauseMenuAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseMenuAction {
    Resume,
    Save,
    Load,
}
