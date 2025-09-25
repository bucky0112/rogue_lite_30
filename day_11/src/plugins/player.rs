use crate::systems::*;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDamagedEvent>()
            .add_systems(Startup, setup)
            .add_systems(PostStartup, spawn_player)
            .add_systems(Update, (movement_system, health_system))
            .add_systems(
                PostUpdate,
                (
                    trigger_player_damage_flash_system,
                    player_damage_flash_tick_system,
                ),
            );
    }
}
