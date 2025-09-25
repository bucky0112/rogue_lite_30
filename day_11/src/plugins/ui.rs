use crate::systems::*;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player_health_ui)
            .add_systems(Update, (spawn_enemy_health_bars, update_player_health_ui))
            .add_systems(
                PostUpdate,
                (
                    update_enemy_health_bar_positions,
                    update_enemy_health_bar_fill,
                ),
            );
    }
}
