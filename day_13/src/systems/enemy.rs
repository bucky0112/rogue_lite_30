use super::health::{PlayerDamagedEvent, PlayerRespawnedEvent};
use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    existing_slimes: Query<Entity, With<Slime>>,
) {
    if !existing_slimes.is_empty() {
        return;
    }

    spawn_slime_internal(&mut commands, asset_server.as_ref(), &floor_query);
}

pub fn spawn_cyclops(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    existing_cyclops: Query<Entity, With<Cyclops>>,
) {
    if !existing_cyclops.is_empty() {
        return;
    }

    spawn_cyclops_internal(&mut commands, asset_server.as_ref(), &floor_query);
}

fn spawn_slime_internal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    floor_query: &Query<(&Transform, &RoomTile), Without<Enemy>>,
) {
    let spawn_details = find_floor_spawn(floor_query, FloorSpawnPreference::LeftMost);

    let (spawn_position, patrol_origin, patrol_range, initial_direction) = spawn_details
        .map(|info| info.into_values(FloorSpawnPreference::LeftMost))
        .unwrap_or_else(|| {
            (
                Vec3::new(0.0, 0.0, 9.0),
                Vec3::new(0.0, 0.0, 9.0),
                SLIME_PATROL_RANGE,
                1.0,
            )
        });

    commands.spawn((
        Enemy,
        Slime,
        Sprite::from_image(asset_server.load("characters/enemies/slime.png")),
        Transform::from_translation(spawn_position).with_scale(Vec3::splat(SLIME_SCALE)),
        Health::new(SLIME_HEALTH),
        Attack::new(SLIME_BASE_ATTACK),
        Defense::new(SLIME_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: patrol_origin,
            range: patrol_range,
            direction: initial_direction,
        },
        EnemyAlert {
            trigger_radius: SLIME_ALERT_RADIUS,
            leash_radius: SLIME_LEASH_RADIUS,
        },
        EnemySpeeds {
            patrol: SLIME_PATROL_SPEED,
            chase: SLIME_CHASE_SPEED,
        },
        EnemyAttack {
            radius: SLIME_ATTACK_RADIUS,
            cooldown: {
                let mut timer = Timer::from_seconds(SLIME_ATTACK_COOLDOWN, TimerMode::Repeating);
                timer.set_elapsed(timer.duration());
                timer
            },
        },
    ));
}

fn spawn_cyclops_internal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    floor_query: &Query<(&Transform, &RoomTile), Without<Enemy>>,
) {
    let spawn_details = find_floor_spawn(floor_query, FloorSpawnPreference::RightMost);

    let (spawn_position, patrol_origin, patrol_range, initial_direction) = spawn_details
        .map(|info| info.into_values(FloorSpawnPreference::RightMost))
        .unwrap_or_else(|| {
            (
                Vec3::new(120.0, 0.0, 9.0),
                Vec3::new(120.0, 0.0, 9.0),
                CYCLOPS_PATROL_RANGE,
                -1.0,
            )
        });

    commands.spawn((
        Enemy,
        Cyclops,
        Sprite::from_image(asset_server.load("characters/enemies/cyclops.png")),
        Transform::from_translation(spawn_position).with_scale(Vec3::splat(CYCLOPS_SCALE)),
        Health::new(CYCLOPS_HEALTH),
        Attack::new(CYCLOPS_BASE_ATTACK),
        Defense::new(CYCLOPS_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: patrol_origin,
            range: patrol_range,
            direction: initial_direction,
        },
        EnemyAlert {
            trigger_radius: CYCLOPS_ALERT_RADIUS,
            leash_radius: CYCLOPS_LEASH_RADIUS,
        },
        EnemySpeeds {
            patrol: CYCLOPS_PATROL_SPEED,
            chase: CYCLOPS_CHASE_SPEED,
        },
        CyclopsCharge {
            windup: Timer::from_seconds(CYCLOPS_WINDUP_SECONDS, TimerMode::Once),
            charge: Timer::from_seconds(CYCLOPS_CHARGE_SECONDS, TimerMode::Once),
            cooldown: {
                let mut timer = Timer::from_seconds(CYCLOPS_COOLDOWN_SECONDS, TimerMode::Once);
                timer.set_elapsed(timer.duration());
                timer
            },
            facing: Vec2::X,
            ready: true,
        },
    ));
}

pub fn slime_ai_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Slime>, Without<PlayerDead>)>,
    mut slime_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
        ),
        (With<Slime>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    process_enemy_ai::<Slime>(time.as_ref(), player_position, &mut slime_query);
}

pub fn cyclops_ai_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Cyclops>, Without<PlayerDead>)>,
    mut cyclops_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
            &mut CyclopsCharge,
        ),
        (With<Cyclops>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    let delta = time.delta();
    let delta_secs = time.delta_secs();

    for (mut transform, mut ai_state, mut patrol, alert, speeds, mut charge) in &mut cyclops_query {
        if !charge.ready {
            charge.cooldown.tick(delta);
            if charge.cooldown.finished() {
                charge.ready = true;
            }
        }

        let (min_x, max_x) = patrol.bounds();

        if let Some(player_pos) = player_position {
            let to_player = player_pos - transform.translation;
            let distance_to_player = to_player.truncate().length();

            match ai_state.state {
                EnemyBehaviorState::WindUp => {
                    charge.windup.tick(delta);
                    let desired = to_player.truncate().normalize_or_zero();
                    if desired != Vec2::ZERO {
                        charge.facing = desired;
                    }

                    if charge.windup.finished() {
                        charge.charge.reset();
                        ai_state.state = EnemyBehaviorState::Charging;
                        if charge.facing == Vec2::ZERO {
                            charge.facing = Vec2::new(patrol.direction, 0.0);
                        }
                    }

                    transform.translation.z = patrol.origin.z;

                    continue;
                }
                EnemyBehaviorState::Charging => {
                    charge.charge.tick(delta);
                    let direction = if charge.facing.length_squared() > 0.0 {
                        charge.facing.normalize()
                    } else {
                        Vec2::new(patrol.direction, 0.0)
                    };
                    let displacement =
                        direction * speeds.chase * CYCLOPS_CHARGE_MULTIPLIER * delta_secs;

                    transform.translation.x =
                        (transform.translation.x + displacement.x).clamp(min_x, max_x);
                    transform.translation.y = patrol.origin.y;
                    transform.translation.z = patrol.origin.z;

                    if charge.charge.finished()
                        || transform.translation.x >= max_x - 1.0
                        || transform.translation.x <= min_x + 1.0
                    {
                        ai_state.state = EnemyBehaviorState::Patrolling;
                        patrol.direction = if direction.x < 0.0 { -1.0 } else { 1.0 };
                        charge.cooldown.reset();
                        charge.ready = false;
                    }

                    continue;
                }
                _ => {}
            }

            if distance_to_player <= alert.trigger_radius {
                if charge.ready {
                    ai_state.state = EnemyBehaviorState::WindUp;
                    charge.windup.reset();
                    charge.facing = to_player.truncate().normalize_or_zero();
                    if charge.facing == Vec2::ZERO {
                        charge.facing = Vec2::new(patrol.direction, 0.0);
                    }
                    transform.translation.z = patrol.origin.z;

                    continue;
                } else {
                    ai_state.state = EnemyBehaviorState::Chasing;
                    let direction = to_player.truncate().normalize_or_zero();
                    let velocity = direction * speeds.chase * delta_secs;

                    transform.translation.x =
                        (transform.translation.x + velocity.x).clamp(min_x, max_x);
                    transform.translation.y = patrol.origin.y;
                    transform.translation.z = patrol.origin.z;

                    if velocity.x.abs() > f32::EPSILON {
                        patrol.direction = velocity.x.signum();
                    }

                    continue;
                }
            } else if distance_to_player > alert.leash_radius {
                ai_state.state = EnemyBehaviorState::Patrolling;
            }
        } else {
            ai_state.state = EnemyBehaviorState::Patrolling;
        }

        let delta_move = patrol.direction * speeds.patrol * delta_secs;
        transform.translation.x += delta_move;

        if transform.translation.x > max_x {
            transform.translation.x = max_x;
            patrol.direction = -1.0;
        } else if transform.translation.x < min_x {
            transform.translation.x = min_x;
            patrol.direction = 1.0;
        }

        transform.translation.y = patrol.origin.y;
        transform.translation.z = patrol.origin.z;
    }
}

pub fn enemy_contact_attack_system(
    time: Res<Time>,
    mut player_query: Query<
        (&Transform, &mut Health, Option<&Defense>),
        (With<Player>, Without<PlayerDead>),
    >,
    mut attacker_query: Query<
        (&Transform, &mut EnemyAttack, &Attack),
        (With<Slime>, Without<EnemyDeathEffect>),
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
) {
    let mut player_iter = player_query.iter_mut();
    let Some((player_transform, mut health, defense)) = player_iter.next() else {
        return;
    };

    let player_position = player_transform.translation.truncate();

    let defense_value = defense.map(|value| value.value());

    for (attacker_transform, mut attack, attack_stat) in &mut attacker_query {
        attack.cooldown.tick(time.delta());

        let distance = attacker_transform
            .translation
            .truncate()
            .distance(player_position);

        if distance <= attack.radius && attack.cooldown.finished() {
            let damage = compute_damage(attack_stat.value(), defense_value);
            let new_health = (health.current - damage).max(0);

            if new_health != health.current {
                health.current = new_health;
                info!(
                    "史萊姆攻擊造成 {} 傷害，玩家剩餘 HP: {}",
                    damage, health.current
                );

                damage_events.write(PlayerDamagedEvent {
                    damage,
                    remaining_health: health.current,
                });
            }

            attack.cooldown.reset();
        }
    }
}

pub fn despawn_dead_enemies_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, Option<&EnemyDeathEffect>, &mut Sprite), With<Enemy>>,
) {
    for (entity, health, death_effect, mut sprite) in &mut query {
        if health.current > 0 {
            continue;
        }

        if death_effect.is_some() {
            continue;
        }

        sprite.color.set_alpha(1.0);

        commands.entity(entity).insert(EnemyDeathEffect {
            timer: Timer::from_seconds(ENEMY_DEATH_FADE_SECONDS, TimerMode::Once),
        });

        info!("敵人被擊倒，開始淡出");
    }
}

pub fn enemy_death_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyDeathEffect, &mut Sprite)>,
) {
    for (entity, mut effect, mut sprite) in &mut query {
        effect.timer.tick(time.delta());

        let duration = effect.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (effect.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
        sprite.color.set_alpha(1.0 - progress);

        if effect.timer.finished() {
            info!("敵人淡出結束，從場景中移除");
            commands.entity(entity).despawn();
        }
    }
}

pub fn reset_enemies_on_player_respawn(
    mut commands: Commands,
    mut respawn_events: EventReader<PlayerRespawnedEvent>,
    mut enemy_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Health,
            Option<&mut EnemyPatrol>,
            Option<&mut EnemyAIState>,
            Option<&mut EnemyAttack>,
            Option<&mut CyclopsCharge>,
            Option<&EnemyDeathEffect>,
            Option<&mut Sprite>,
        ),
        With<Enemy>,
    >,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    slime_entities: Query<Entity, With<Slime>>,
    cyclops_entities: Query<Entity, With<Cyclops>>,
) {
    let mut triggered = false;
    for _ in respawn_events.read() {
        triggered = true;
    }

    if !triggered {
        return;
    }

    for (
        entity,
        mut transform,
        mut health,
        patrol,
        ai_state,
        attack,
        charge,
        death_effect,
        sprite,
    ) in &mut enemy_query
    {
        health.current = health.max;

        if let Some(mut patrol) = patrol {
            transform.translation = patrol.origin;
            patrol.direction = if patrol.direction >= 0.0 { 1.0 } else { -1.0 };
        }

        if let Some(mut ai_state) = ai_state {
            ai_state.state = EnemyBehaviorState::Patrolling;
        }

        if let Some(mut attack) = attack {
            let duration = attack.cooldown.duration();
            attack.cooldown.reset();
            attack.cooldown.set_elapsed(duration);
        }

        if let Some(mut charge) = charge {
            charge.windup.reset();
            charge.charge.reset();
            let cooldown_duration = charge.cooldown.duration();
            charge.cooldown.reset();
            charge.cooldown.set_elapsed(cooldown_duration);
            charge.ready = true;
            charge.facing = Vec2::X;
        }

        if let Some(mut sprite) = sprite {
            sprite.color = Color::WHITE;
        }

        if death_effect.is_some() {
            commands.entity(entity).remove::<EnemyDeathEffect>();
        }
    }

    if slime_entities.iter().next().is_none() {
        spawn_slime_internal(&mut commands, asset_server.as_ref(), &floor_query);
    }

    if cyclops_entities.iter().next().is_none() {
        spawn_cyclops_internal(&mut commands, asset_server.as_ref(), &floor_query);
    }

    info!("敵人群已重置並回復至初始狀態");
}

fn process_enemy_ai<M: Component>(
    time: &Time,
    player_position: Option<Vec3>,
    query: &mut Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
        ),
        (With<M>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    for (mut transform, mut ai_state, mut patrol, alert, speeds) in query.iter_mut() {
        if let Some(player_pos) = player_position {
            let to_player = player_pos - transform.translation;
            let distance_to_player = to_player.truncate().length();

            match ai_state.state {
                EnemyBehaviorState::Patrolling => {
                    if distance_to_player <= alert.trigger_radius {
                        ai_state.state = EnemyBehaviorState::Chasing;
                    }
                }
                EnemyBehaviorState::Chasing => {
                    if distance_to_player > alert.leash_radius {
                        ai_state.state = EnemyBehaviorState::Patrolling;
                        patrol.direction = if transform.translation.x >= patrol.origin.x {
                            -1.0
                        } else {
                            1.0
                        };
                    }
                }
                EnemyBehaviorState::WindUp | EnemyBehaviorState::Charging => {
                    ai_state.state = EnemyBehaviorState::Patrolling;
                }
            }

            if ai_state.state == EnemyBehaviorState::Chasing {
                let direction = to_player.truncate().normalize_or_zero();
                let velocity = direction * speeds.chase * time.delta_secs();

                transform.translation.x += velocity.x;
                transform.translation.y += velocity.y;

                transform.translation.z = patrol.origin.z;

                continue;
            }
        } else {
            ai_state.state = EnemyBehaviorState::Patrolling;
        }

        let (min_x, max_x) = patrol.bounds();
        let delta = patrol.direction * speeds.patrol * time.delta_secs();
        transform.translation.x += delta;

        if transform.translation.x > max_x {
            transform.translation.x = max_x;
            patrol.direction = -1.0;
        } else if transform.translation.x < min_x {
            transform.translation.x = min_x;
            patrol.direction = 1.0;
        }

        transform.translation.y = patrol.origin.y;
        transform.translation.z = patrol.origin.z;
    }
}

enum FloorSpawnPreference {
    LeftMost,
    RightMost,
}

struct FloorSpawnInfo {
    position: Vec3,
    min_x: f32,
    max_x: f32,
}

impl FloorSpawnInfo {
    fn into_values(self, preference: FloorSpawnPreference) -> (Vec3, Vec3, f32, f32) {
        let spawn_position = self.position;
        let patrol_center_x = (self.min_x + self.max_x) * 0.5;
        let max_range = match preference {
            FloorSpawnPreference::LeftMost => SLIME_PATROL_RANGE,
            FloorSpawnPreference::RightMost => CYCLOPS_PATROL_RANGE,
        };
        let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
        let half_width = (self.max_x - self.min_x) * 0.5;
        let margin = tile_span * 0.1;
        let patrol_range = (half_width - margin).max(tile_span * 0.25).min(max_range);
        let direction = match preference {
            FloorSpawnPreference::LeftMost => 1.0,
            FloorSpawnPreference::RightMost => -1.0,
        };

        (
            spawn_position,
            Vec3::new(patrol_center_x, spawn_position.y, spawn_position.z),
            patrol_range,
            direction,
        )
    }
}

fn find_floor_spawn(
    floor_query: &Query<(&Transform, &RoomTile), Without<Enemy>>,
    preference: FloorSpawnPreference,
) -> Option<FloorSpawnInfo> {
    let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
    let mut rows: HashMap<i32, (f32, f32)> = HashMap::new();
    let mut best: Option<(Vec3, i32)> = None;

    for (transform, tile) in floor_query.iter() {
        if tile.tile_type != RoomTileType::Floor {
            continue;
        }

        let row_key = (transform.translation.y / tile_span).round() as i32;
        let entry = rows
            .entry(row_key)
            .or_insert((transform.translation.x, transform.translation.x));
        entry.0 = entry.0.min(transform.translation.x);
        entry.1 = entry.1.max(transform.translation.x);

        let candidate = Vec3::new(transform.translation.x, transform.translation.y, 9.0);

        best = match best {
            Some((current_pos, current_row)) => match preference {
                FloorSpawnPreference::LeftMost => {
                    if candidate.x < current_pos.x {
                        Some((candidate, row_key))
                    } else {
                        Some((current_pos, current_row))
                    }
                }
                FloorSpawnPreference::RightMost => {
                    if candidate.x > current_pos.x {
                        Some((candidate, row_key))
                    } else {
                        Some((current_pos, current_row))
                    }
                }
            },
            None => Some((candidate, row_key)),
        };
    }

    best.and_then(|(position, row_key)| {
        rows.get(&row_key).map(|(min_x, max_x)| FloorSpawnInfo {
            position,
            min_x: *min_x,
            max_x: *max_x,
        })
    })
}
