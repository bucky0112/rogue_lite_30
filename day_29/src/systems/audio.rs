use crate::components::MainThemeMusic;
use crate::constants::{GAMEPLAY_MUSIC_VOLUME, MENU_MUSIC_VOLUME};
use crate::resources::{BackgroundMusicState, GamePhase, GameSession, SoundEffects};
use crate::systems::{
    BossWizardSpellCastEvent, DoorStateChangedEvent, EnemyAttackHitEvent, MenuClickEvent,
    PlayerLevelUpEvent, PlayerMeleeAttackEvent, PlayerPickupEvent, PlayerPoisonDamageEvent,
};
use bevy::audio::{AudioPlayer, AudioSink, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

pub fn initialize_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    session: Res<GameSession>,
) {
    let background_music = asset_server.load("sounds/bgm/Space-Cadet.ogg");

    commands.insert_resource(SoundEffects {
        player_attack: asset_server.load("sounds/player/hit.ogg"),
        player_pickup: asset_server.load("sounds/player/pickup.ogg"),
        player_upgrade: asset_server.load("sounds/player/upgrade.ogg"),
        player_hurt: asset_server.load("sounds/player/hurt.ogg"),
        enemy_attack: asset_server.load("sounds/enemy/hit.ogg"),
        door_open: asset_server.load("sounds/sfx/door_open.ogg"),
        door_close: asset_server.load("sounds/sfx/door_close.ogg"),
        boss_wizard_spell: asset_server.load("sounds/enemy/explosion3.ogg"),
        ui_click: asset_server.load("sounds/ui/click.ogg"),
    });

    let initial_phase = session.phase();
    commands.insert_resource(BackgroundMusicState {
        current_phase: Some(initial_phase),
    });

    let mut settings = PlaybackSettings::LOOP.with_volume(Volume::Linear(match initial_phase {
        GamePhase::MainMenu => MENU_MUSIC_VOLUME,
        GamePhase::Playing => GAMEPLAY_MUSIC_VOLUME,
        GamePhase::Paused => GAMEPLAY_MUSIC_VOLUME,
    }));

    if matches!(initial_phase, GamePhase::Paused) {
        settings.paused = true;
    }

    commands.spawn((MainThemeMusic, AudioPlayer::new(background_music), settings));
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

pub fn play_menu_click_sound(
    mut commands: Commands,
    mut events: EventReader<MenuClickEvent>,
    sounds: Res<SoundEffects>,
) {
    if events.read().next().is_some() {
        spawn_one_shot(&mut commands, &sounds.ui_click);
    }
}

pub fn play_boss_wizard_spell_sound(
    mut commands: Commands,
    mut events: EventReader<BossWizardSpellCastEvent>,
    sounds: Res<SoundEffects>,
) {
    if events.read().next().is_some() {
        spawn_one_shot(&mut commands, &sounds.boss_wizard_spell);
    }
}

pub fn update_background_music_volume(
    session: Res<GameSession>,
    mut state: ResMut<BackgroundMusicState>,
    mut music_query: Query<&mut AudioSink, With<MainThemeMusic>>,
) {
    let Ok(mut sink) = music_query.get_single_mut() else {
        return;
    };

    let target_phase = session.phase();
    if state.current_phase == Some(target_phase) && !session.is_changed() {
        return;
    }

    match target_phase {
        GamePhase::MainMenu => {
            sink.unmute();
            sink.set_volume(Volume::Linear(MENU_MUSIC_VOLUME));
            if sink.is_paused() {
                sink.play();
            }
        }
        GamePhase::Playing => {
            sink.unmute();
            sink.set_volume(Volume::Linear(GAMEPLAY_MUSIC_VOLUME));
            if sink.is_paused() {
                sink.play();
            }
        }
        GamePhase::Paused => {
            sink.set_volume(Volume::Linear(GAMEPLAY_MUSIC_VOLUME));
            if !sink.is_paused() {
                sink.pause();
            }
        }
    }

    state.current_phase = Some(target_phase);
}

fn spawn_one_shot(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle.clone()), PlaybackSettings::DESPAWN));
}
