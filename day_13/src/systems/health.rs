use crate::components::*;
use crate::constants::*;
use crate::resources::{EntranceLocation, PlayerDeathState};
use bevy::prelude::*;

#[derive(Event, Clone, Copy)]
pub struct PlayerDamagedEvent {
    pub damage: i32,
    pub remaining_health: i32,
}

#[derive(Event, Clone, Copy)]
pub struct PlayerDiedEvent;

#[derive(Event, Clone, Copy)]
pub struct PlayerRespawnedEvent;

pub fn health_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, Option<&PlayerDead>), With<Player>>,
    mut death_events: EventWriter<PlayerDiedEvent>,
) {
    for (entity, health, dead_marker) in &mut query {
        if health.current > 0 {
            continue;
        }

        if dead_marker.is_some() {
            continue;
        }

        commands.entity(entity).insert(PlayerDead);
        death_events.write(PlayerDiedEvent);
        info!("玩家死亡！");
    }
}

pub fn start_player_death_sequence_system(
    mut commands: Commands,
    mut death_events: EventReader<PlayerDiedEvent>,
    mut death_state: ResMut<PlayerDeathState>,
    mut player_query: Query<
        (
            Entity,
            &mut Sprite,
            Option<Mut<DamageFlash>>,
            Option<Mut<Velocity>>,
            Option<Mut<InputVector>>,
        ),
        With<Player>,
    >,
) {
    let mut triggered = false;
    for _ in death_events.read() {
        triggered = true;
    }

    if !triggered || death_state.is_active() {
        return;
    }

    let Ok((entity, mut sprite, damage_flash, velocity, input_vector)) = player_query.single_mut()
    else {
        return;
    };

    if let Some(mut vel) = velocity {
        vel.x = 0.0;
        vel.y = 0.0;
    }

    if let Some(mut input) = input_vector {
        input.0 = Vec2::ZERO;
    }

    if damage_flash.is_some() {
        commands.entity(entity).remove::<DamageFlash>();
    }

    sprite.color = Color::WHITE;

    death_state.start(PLAYER_DEATH_DISPLAY_SECONDS);
}

pub fn player_respawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut death_state: ResMut<PlayerDeathState>,
    entrance_location: Option<Res<EntranceLocation>>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Health,
            Option<Mut<Velocity>>,
            Option<Mut<InputVector>>,
            &mut Sprite,
        ),
        With<Player>,
    >,
    mut respawn_events: EventWriter<PlayerRespawnedEvent>,
) {
    let Some(timer) = death_state.timer.as_mut() else {
        return;
    };

    if !timer.tick(time.delta()).finished() {
        return;
    }

    let Ok((entity, mut transform, mut health, velocity, input_vector, mut sprite)) =
        player_query.single_mut()
    else {
        return;
    };

    let spawn_position = entrance_location
        .map(|location| location.position)
        .unwrap_or_else(|| Vec3::new(0.0, -ROOM_TILE_SIZE * PLAYER_SCALE * 3.0, 10.0));

    transform.translation = spawn_position;

    if let Some(mut vel) = velocity {
        vel.x = 0.0;
        vel.y = 0.0;
    }

    if let Some(mut input) = input_vector {
        input.0 = Vec2::ZERO;
    }

    health.current = health.max;
    sprite.color = Color::WHITE;

    commands.entity(entity).remove::<PlayerDead>();

    death_state.clear_timer();

    respawn_events.write(PlayerRespawnedEvent);
    info!("玩家已在出生點復活！");
}

pub fn trigger_player_damage_flash_system(
    mut commands: Commands,
    mut events: EventReader<PlayerDamagedEvent>,
    mut player_query: Query<(Entity, &mut Sprite, Option<Mut<DamageFlash>>), With<Player>>,
) {
    let mut last_event = None;
    for event in events.read() {
        last_event = Some(*event);
    }

    let Some(event) = last_event else {
        return;
    };

    let mut player_iter = player_query.iter_mut();
    let Some((entity, mut sprite, maybe_flash)) = player_iter.next() else {
        return;
    };

    let color = Color::srgba(
        PLAYER_DAMAGE_FLASH_COLOR[0],
        PLAYER_DAMAGE_FLASH_COLOR[1],
        PLAYER_DAMAGE_FLASH_COLOR[2],
        PLAYER_DAMAGE_FLASH_COLOR[3],
    );

    sprite.color = color;

    match maybe_flash {
        Some(mut flash) => {
            flash.highlight_color = color;
            flash.restart(PLAYER_DAMAGE_FLASH_COUNT);
        }
        None => {
            commands.entity(entity).insert(DamageFlash::new(
                PLAYER_DAMAGE_FLASH_COUNT,
                PLAYER_DAMAGE_FLASH_INTERVAL,
                color,
            ));
        }
    }

    debug!(
        "玩家受到了 {} 點傷害，剩餘 HP: {}",
        event.damage, event.remaining_health
    );
}

pub fn player_damage_flash_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut DamageFlash, &mut Sprite), With<Player>>,
) {
    let mut player_iter = player_query.iter_mut();
    let Some((entity, mut flash, mut sprite)) = player_iter.next() else {
        return;
    };

    if flash.timer.tick(time.delta()).just_finished() {
        if flash.show_highlight {
            flash.show_highlight = false;
            sprite.color = Color::WHITE;
            flash.flashes_remaining = flash.flashes_remaining.saturating_sub(1);

            if flash.flashes_remaining == 0 {
                commands.entity(entity).remove::<DamageFlash>();
            }
        } else {
            flash.show_highlight = true;
            sprite.color = flash.highlight_color;
        }
    }
}
