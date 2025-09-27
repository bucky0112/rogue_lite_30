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
) {
    for _event in attack_events.read() {
        let mut started_attack = false;
        for mut swing in &mut weapon_query {
            // Only start new attack if not already attacking
            if swing.timer.finished() {
                swing.timer.reset();
                swing.from_angle = -PI / 4.0; // -45 degrees
                swing.to_angle = PI / 4.0; // +45 degrees
                started_attack = true;
            }
        }

        if started_attack {
            melee_events.write(PlayerMeleeAttackEvent);
        }
    }
}

pub fn update_weapon_offset_system(
    player_query: Query<&PlayerFacing, (With<Player>, Changed<PlayerFacing>)>,
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
    if let Ok(facing) = player_query.single() {
        for (mut offset, mut transform, mut sprite, weapon_sprites) in &mut weapon_query {
            let (position, angle, is_left_side) =
                calculate_weapon_position_and_rotation(&facing.direction);

            // Update weapon offset data
            offset.position = position;
            offset.base_angle = angle;

            // Update transform position
            transform.translation.x = position.x;
            transform.translation.y = position.y;

            // Switch sprite based on facing side
            sprite.image = if is_left_side {
                weapon_sprites.left_sprite.clone()
            } else {
                weapon_sprites.right_sprite.clone()
            };

            // Set base rotation
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
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

fn calculate_weapon_position_and_rotation(facing_direction: &Vec2) -> (Vec2, f32, bool) {
    let angle = facing_direction.y.atan2(facing_direction.x);
    let octant = get_direction_octant(angle);

    let hand_offset = match octant {
        0 => Vec2::new(8.0, 2.0),   // 向右
        1 => Vec2::new(6.0, 6.0),   // 右上
        2 => Vec2::new(-2.0, -4.0), // 向上
        3 => Vec2::new(-6.0, 6.0),  // 左上
        4 => Vec2::new(-8.0, 2.0),  // 向左
        5 => Vec2::new(-6.0, -2.0), // 左下
        6 => Vec2::new(-2.0, -4.0), // 向下
        7 => Vec2::new(6.0, -2.0),  // 右下
        _ => Vec2::new(8.0, 2.0),
    };

    let is_left_side = matches!(octant, 3 | 4 | 5); // 左上、左、左下
    (hand_offset, angle, is_left_side)
}

fn get_direction_octant(angle: f32) -> usize {
    let normalized_angle = if angle < 0.0 { angle + 2.0 * PI } else { angle };
    let octant_size = PI / 4.0;
    let octant = ((normalized_angle + octant_size / 2.0) / octant_size) as usize;
    octant % 8
}

fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    from + (to - from) * t
}

pub fn player_melee_attack_system(
    mut attack_events: EventReader<PlayerMeleeAttackEvent>,
    player_query: Query<(&Transform, &PlayerFacing, &Attack), (With<Player>, Without<PlayerDead>)>,
    mut enemy_query: Query<(&Transform, &mut Health, Option<&Defense>), With<Enemy>>,
) {
    let mut attack_count = 0;
    for _ in attack_events.read() {
        attack_count += 1;
    }

    if attack_count == 0 {
        return;
    }

    let Some((player_transform, facing, attack)) = player_query.iter().next() else {
        return;
    };

    let facing_direction = facing.direction.normalize_or_zero();
    if facing_direction == Vec2::ZERO {
        return;
    }

    let attack_center =
        player_transform.translation.truncate() + facing_direction * PLAYER_ATTACK_REACH;
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
