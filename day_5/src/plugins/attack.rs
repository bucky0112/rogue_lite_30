use bevy::prelude::*;
use crate::systems::*;

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                attack_input_system,
                update_weapon_offset_system,
                update_weapon_swing_animation_system,
            ));
    }
}

