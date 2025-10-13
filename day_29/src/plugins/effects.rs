use crate::systems::*;
use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_hit_spark_system.after(player_melee_attack_system),
                update_hit_spark_system,
                apply_enemy_hit_flash_system.after(player_melee_attack_system),
                update_enemy_hit_flash_system,
                spawn_enemy_death_particles_system.after(player_melee_attack_system),
                update_death_particles_system,
                trigger_camera_shake_on_enemy_hit.after(player_melee_attack_system),
            ),
        );
    }
}
