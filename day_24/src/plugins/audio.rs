use crate::systems::*;
use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sound_effects).add_systems(
            Update,
            (
                play_player_attack_sound,
                play_enemy_attack_sound,
                play_player_pickup_sound,
                play_player_level_up_sound,
                play_player_poison_damage_sound,
                play_door_state_sound,
            ),
        );
    }
}
