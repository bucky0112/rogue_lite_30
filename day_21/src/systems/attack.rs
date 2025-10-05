use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Event)]
pub struct AttackInputEvent;

#[derive(Event)]
pub struct PlayerMeleeAttackEvent;

pub fn attack_input_system(
    mut attack_events: EventReader<AttackInputEvent>,
    mut weapon_query: Query<&mut WeaponSwing, With<Weapon>>,
    mut melee_events: EventWriter<PlayerMeleeAttackEvent>,
    mut stamina_query: Query<&mut Stamina, (With<Player>, Without<PlayerDead>)>,
) {
    let mut requested = false;
    for _ in attack_events.read() {
        requested = true;
    }

    if !requested {
        return;
    }

    let Some(mut stamina) = stamina_query.iter_mut().next() else {
        return;
    };

    let mut started_attack = false;
    let mut spent_stamina = false;

    for mut swing in &mut weapon_query {
        if !swing.timer.finished() {
            continue;
        }

        if !spent_stamina {
            if !stamina.spend(PLAYER_ATTACK_STAMINA_COST) {
                info!("耐力不足，攻擊動作取消。");
                return;
            }

            spent_stamina = true;
        }

        swing.timer.reset();
        swing.from_angle = -PI / 4.0; // -45 degrees
        swing.to_angle = PI / 4.0; // +45 degrees
        started_attack = true;
    }

    if started_attack {
        melee_events.write(PlayerMeleeAttackEvent);
    }
}

pub fn update_weapon_offset_system(
    mut weapon_query: Query<
        (
            &mut WeaponOffset,
            &mut Transform,
            &mut Sprite,
            &WeaponSprites,
        ),
        With<Weapon>,
    >,
) {
    for (mut offset, mut transform, mut sprite, weapon_sprites) in &mut weapon_query {
        let position = Vec2::new(WEAPON_IDLE_OFFSET_X, WEAPON_IDLE_OFFSET_Y);

        // Update weapon offset data
        offset.position = position;
        offset.base_angle = 0.0;

        // Update transform position
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.translation.z = WEAPON_Z;

        // 劍固定在玩家右側，維持右手持有的貼圖
        sprite.image = weapon_sprites.right_sprite.clone();

        // Set base rotation
        transform.rotation = Quat::from_rotation_z(0.0);
    }
}

pub fn update_attack_reticle_system(
    mut reticle_query: Query<(&mut AttackReticle, &mut Transform), Without<Player>>,
    player_query: Query<
        (&Transform, &PlayerFacing),
        (With<Player>, Without<PlayerDead>, Without<AttackReticle>),
    >,
) {
    let Some((player_transform, facing)) = player_query.iter().next() else {
        return;
    };

    let Some((mut reticle, mut transform)) = reticle_query.iter_mut().next() else {
        return;
    };

    let raw_direction = facing.direction;
    if raw_direction.length_squared() > INPUT_DEADZONE * INPUT_DEADZONE {
        let x_abs = raw_direction.x.abs();
        let y_abs = raw_direction.y.abs();

        let axis_direction = if x_abs >= y_abs {
            Vec2::new(raw_direction.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, raw_direction.y.signum())
        };

        if axis_direction != Vec2::ZERO {
            reticle.last_direction = axis_direction;
        }
    }

    if reticle.last_direction == Vec2::ZERO {
        reticle.last_direction = Vec2::new(1.0, 0.0);
    }

    let player_position = player_transform.translation;
    let offset = reticle.last_direction * ATTACK_RETICLE_DISTANCE;

    transform.translation.x = player_position.x + offset.x;
    transform.translation.y = player_position.y + offset.y;
    transform.translation.z = player_position.z + ATTACK_RETICLE_Z_OFFSET;
}

pub fn update_weapon_swing_animation_system(
    mut weapon_query: Query<(&mut WeaponSwing, &mut Transform, &WeaponOffset), With<Weapon>>,
    time: Res<Time>,
) {
    for (mut swing, mut transform, offset) in &mut weapon_query {
        if !swing.timer.finished() {
            swing.timer.tick(time.delta());

            if swing.timer.finished() {
                swing.from_angle = 0.0;
                swing.to_angle = 0.0;
                transform.rotation = Quat::from_rotation_z(offset.base_angle);
                continue;
            }

            let progress = swing.timer.elapsed_secs() / swing.timer.duration().as_secs_f32();
            let current_angle = lerp_angle(swing.from_angle, swing.to_angle, progress);
            let adjusted_angle = offset.base_angle + current_angle;

            transform.rotation = Quat::from_rotation_z(adjusted_angle);
        } else {
            transform.rotation = Quat::from_rotation_z(offset.base_angle);
        }
    }
}

fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    from + (to - from) * t
}

pub fn player_melee_attack_system(
    mut attack_events: EventReader<PlayerMeleeAttackEvent>,
    player_query: Query<&Attack, (With<Player>, Without<PlayerDead>)>,
    reticle_query: Query<(&Transform, &AttackReticle), Without<Player>>,
    mut enemy_query: Query<(&Transform, &mut Health, Option<&Defense>), With<Enemy>>,
) {
    let mut attack_count = 0;
    for _ in attack_events.read() {
        attack_count += 1;
    }

    if attack_count == 0 {
        return;
    }

    let Some(attack) = player_query.iter().next() else {
        return;
    };

    let Some((reticle_transform, reticle)) = reticle_query.iter().next() else {
        return;
    };

    let facing_direction = reticle.last_direction.normalize_or_zero();
    if facing_direction == Vec2::ZERO {
        return;
    }

    let attack_center = reticle_transform.translation.truncate();
    let total_attack = attack.value() * attack_count as i32;

    for (enemy_transform, mut health, defense) in &mut enemy_query {
        if health.current <= 0 {
            continue;
        }

        let to_enemy = enemy_transform.translation.truncate() - attack_center;
        let distance = to_enemy.length();

        if distance > PLAYER_ATTACK_RADIUS || distance == 0.0 {
            continue;
        }

        let direction_to_enemy = to_enemy / distance;

        if facing_direction.dot(direction_to_enemy) < PLAYER_ATTACK_FACING_COS_THRESHOLD {
            continue;
        }

        let defense_value = defense.map(|value| value.value());
        let damage = compute_damage(total_attack, defense_value);
        let new_health = (health.current - damage).max(0);
        if new_health != health.current {
            health.current = new_health;
            info!(
                "玩家攻擊造成 {} 傷害，敵人剩餘 HP: {}",
                damage, health.current
            );
        }
    }
}
