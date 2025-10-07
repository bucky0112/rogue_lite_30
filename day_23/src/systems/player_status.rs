use crate::components::*;
use crate::constants::*;
use crate::systems::health::PlayerDamagedEvent;
use bevy::prelude::*;

#[derive(Event, Clone, Copy)]
pub struct PlayerPoisonDamageEvent;

/// 當玩家沒有連續出招時，耐力會逐步回復
pub fn player_stamina_regen_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Stamina, (With<Player>, Without<PlayerDead>)>,
) {
    let Some(mut stamina) = query.iter_mut().next() else {
        return;
    };

    if keyboard_input.pressed(KeyCode::Space) {
        return;
    }

    stamina.regen(time.delta_secs());
}

/// 玩家中毒時定期扣血
pub fn player_poison_tick_system(
    time: Res<Time>,
    mut query: Query<(&mut Health, &mut Poisoned), (With<Player>, Without<PlayerDead>)>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut poison_damage_events: EventWriter<PlayerPoisonDamageEvent>,
) {
    let Some((mut health, mut poisoned)) = query.iter_mut().next() else {
        return;
    };

    if !poisoned.tick_timer.tick(time.delta()).just_finished() {
        return;
    }

    let damage = poisoned.damage_per_tick;
    let new_health = (health.current - damage).max(0);

    if new_health == health.current {
        return;
    }

    health.current = new_health;
    damage_events.write(PlayerDamagedEvent {
        damage,
        remaining_health: health.current,
    });
    poison_damage_events.write(PlayerPoisonDamageEvent);

    info!(
        "中毒造成 {} 點傷害，玩家剩餘 HP: {}",
        damage, health.current
    );
}

/// 方便除錯：快速套用或解除中毒／補滿耐力
pub fn player_status_debug_shortcuts_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Stamina, Option<&Poisoned>),
        (With<Player>, Without<PlayerDead>),
    >,
) {
    let triggered = keyboard_input.just_pressed(KeyCode::KeyT)
        || keyboard_input.just_pressed(KeyCode::KeyG)
        || keyboard_input.just_pressed(KeyCode::KeyY);

    if !triggered {
        return;
    }

    let Some((entity, mut stamina, poison_state)) = player_query.iter_mut().next() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        stamina.refill();
        info!("使用耐力藥水，耐力已回滿！");
    }

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        if poison_state.is_some() {
            commands.entity(entity).remove::<Poisoned>();
            info!("喝下解毒藥水，狀態已解除。");
        } else {
            info!("目前沒有中毒狀態，解毒藥水沒有作用。");
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyY) {
        if poison_state.is_none() {
            commands.entity(entity).insert(Poisoned::new(
                PLAYER_POISON_TICK_SECONDS,
                PLAYER_POISON_TICK_DAMAGE,
            ));
            info!("測試用：玩家已被施加中毒效果。");
        } else {
            info!("玩家已經是中毒狀態，請先使用解毒藥水。");
        }
    }
}
