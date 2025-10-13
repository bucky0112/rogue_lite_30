use crate::systems::*;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_player_health_ui, spawn_player_stats_panel))
            .add_systems(
                Update,
                (
                    spawn_enemy_health_bars,
                    update_player_health_ui,
                    update_player_stamina_ui,
                    update_player_status_text,
                    spawn_player_death_screen,
                    update_player_stats_panel,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    update_enemy_health_bar_positions,
                    update_enemy_health_bar_fill,
                    despawn_player_death_screen_on_respawn,
                ),
            );
    }
}
