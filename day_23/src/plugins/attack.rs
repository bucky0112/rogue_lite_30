use crate::systems::*;
use bevy::prelude::*;

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AttackInputEvent>()
            .add_event::<PlayerMeleeAttackEvent>()
            .add_systems(
                Update,
                (
                    attack_input_system,
                    update_attack_reticle_system.after(movement_system),
                    player_melee_attack_system
                        .after(attack_input_system)
                        .after(update_attack_reticle_system),
                    update_weapon_offset_system,
                    update_weapon_swing_animation_system,
                ),
            );
    }
}
