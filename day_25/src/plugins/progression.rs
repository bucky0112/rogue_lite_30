use crate::systems::*;
use bevy::prelude::*;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerLevelUpEvent>().add_systems(
            Update,
            (
                apply_enemy_experience_rewards.after(despawn_dead_enemies_system),
                apply_player_level_up_effects.after(apply_enemy_experience_rewards),
            ),
        );
    }
}
