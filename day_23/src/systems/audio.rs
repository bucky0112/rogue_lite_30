use crate::resources::SoundEffects;
use crate::systems::{
    DoorStateChangedEvent, EnemyAttackHitEvent, PlayerLevelUpEvent, PlayerMeleeAttackEvent,
    PlayerPickupEvent, PlayerPoisonDamageEvent,
};
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;

pub fn load_sound_effects(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundEffects {
        player_attack: asset_server.load("sounds/player/hit.ogg"),
        player_pickup: asset_server.load("sounds/player/pickup.ogg"),
        player_upgrade: asset_server.load("sounds/player/upgrade.ogg"),
        player_hurt: asset_server.load("sounds/player/hurt.ogg"),
        enemy_attack: asset_server.load("sounds/enemy/hit.ogg"),
        door_open: asset_server.load("sounds/sfx/door_open.ogg"),
        door_close: asset_server.load("sounds/sfx/door_close.ogg"),
    });
}

pub fn play_player_attack_sound(
    mut commands: Commands,
    mut events: EventReader<PlayerMeleeAttackEvent>,
    sounds: Res<SoundEffects>,
) {
    let mut triggered = false;
    for _ in events.read() {
        triggered = true;
    }

    if triggered {
        spawn_one_shot(&mut commands, &sounds.player_attack);
    }
}

pub fn play_enemy_attack_sound(
    mut commands: Commands,
    mut events: EventReader<EnemyAttackHitEvent>,
    sounds: Res<SoundEffects>,
) {
    for _ in events.read() {
        spawn_one_shot(&mut commands, &sounds.enemy_attack);
    }
}

pub fn play_player_pickup_sound(
    mut commands: Commands,
    mut events: EventReader<PlayerPickupEvent>,
    sounds: Res<SoundEffects>,
) {
    let mut triggered = false;
    for _ in events.read() {
        triggered = true;
    }

    if triggered {
        spawn_one_shot(&mut commands, &sounds.player_pickup);
    }
}

pub fn play_player_level_up_sound(
    mut commands: Commands,
    mut events: EventReader<PlayerLevelUpEvent>,
    sounds: Res<SoundEffects>,
) {
    let mut triggered = false;
    for _ in events.read() {
        triggered = true;
    }

    if triggered {
        spawn_one_shot(&mut commands, &sounds.player_upgrade);
    }
}

pub fn play_player_poison_damage_sound(
    mut commands: Commands,
    mut events: EventReader<PlayerPoisonDamageEvent>,
    sounds: Res<SoundEffects>,
) {
    let mut triggered = false;
    for _ in events.read() {
        triggered = true;
    }

    if triggered {
        spawn_one_shot(&mut commands, &sounds.player_hurt);
    }
}

pub fn play_door_state_sound(
    mut commands: Commands,
    mut events: EventReader<DoorStateChangedEvent>,
    sounds: Res<SoundEffects>,
) {
    for event in events.read() {
        let handle = if event.is_open {
            &sounds.door_open
        } else {
            &sounds.door_close
        };

        spawn_one_shot(&mut commands, handle);
    }
}

fn spawn_one_shot(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle.clone()), PlaybackSettings::DESPAWN));
}
