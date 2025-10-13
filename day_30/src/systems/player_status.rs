use crate::components::*;
use crate::resources::GameSession;
use crate::systems::health::PlayerDamagedEvent;
use bevy::prelude::*;

#[derive(Event, Clone, Copy)]
pub struct PlayerPoisonDamageEvent;

/// 當玩家沒有連續出招時，耐力會逐步回復
pub fn player_stamina_regen_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    session: Res<GameSession>,
    mut query: Query<&mut Stamina, (With<Player>, Without<PlayerDead>)>,
) {
    if !session.is_playing() {
        return;
    }

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
    session: Res<GameSession>,
    mut query: Query<(&mut Health, &mut Poisoned), (With<Player>, Without<PlayerDead>)>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut poison_damage_events: EventWriter<PlayerPoisonDamageEvent>,
) {
    if !session.is_playing() {
        return;
    }

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

    dev_info!(
        "Poison dealt {} damage; player HP now {}",
        damage,
        health.current
    );
}
