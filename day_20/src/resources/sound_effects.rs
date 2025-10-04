use bevy::audio::AudioSource;
use bevy::prelude::*;

#[derive(Resource)]
pub struct SoundEffects {
    pub player_attack: Handle<AudioSource>,
    pub player_pickup: Handle<AudioSource>,
    pub player_upgrade: Handle<AudioSource>,
    pub player_hurt: Handle<AudioSource>,
    pub enemy_attack: Handle<AudioSource>,
    pub door_open: Handle<AudioSource>,
    pub door_close: Handle<AudioSource>,
}
