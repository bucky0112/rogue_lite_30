use crate::resources::PlayerDeathState;
use crate::systems::*;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>()
            .add_event::<PlayerDamagedEvent>()
            .add_event::<PlayerDiedEvent>()
            .add_event::<PlayerRespawnedEvent>()
            .add_systems(Startup, setup)
            .add_systems(PostStartup, spawn_player)
            .add_systems(
                Update,
                (
                    player_attribute_debug_input_system,
                    player_status_debug_shortcuts_system,
                    movement_system,
                    player_poison_tick_system,
                    player_stamina_regen_system,
                    health_system,
                    start_player_death_sequence_system.after(health_system),
                    player_respawn_system.after(start_player_death_sequence_system),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    trigger_player_damage_flash_system,
                    player_damage_flash_tick_system,
                ),
            );
    }
}
