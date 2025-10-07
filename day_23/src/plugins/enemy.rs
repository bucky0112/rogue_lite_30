use crate::systems::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDefeatedEvent>()
            .add_event::<EnemyAttackHitEvent>()
            .add_systems(
                Update,
                (
                    mimic_ai_system,
                    slime_ai_system,
                    cyclops_ai_system,
                    enemy_contact_attack_system,
                    despawn_dead_enemies_system.after(player_melee_attack_system),
                    reset_enemies_on_player_respawn
                        .after(player_respawn_system)
                        .before(enemy_death_effect_system),
                    enemy_death_effect_system.after(despawn_dead_enemies_system),
                ),
            );
    }
}
