use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;

#[derive(Event, Clone, Copy)]
pub struct PlayerDamagedEvent {
    pub damage: i32,
    pub remaining_health: i32,
}

pub fn health_system(query: Query<&Health, With<Player>>) {
    for health in &query {
        if health.current <= 0 {
            info!("玩家死亡！");
        }
    }
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
