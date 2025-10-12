use crate::systems::*;
use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_audio).add_systems(
            Update,
            (
                update_background_music_volume,
                play_menu_click_sound,
                play_boss_wizard_spell_sound,
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
