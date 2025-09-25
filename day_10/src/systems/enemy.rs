use super::health::PlayerDamagedEvent;
use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile)>,
    existing_slimes: Query<Entity, With<Slime>>,
) {
    if !existing_slimes.is_empty() {
        return;
    }

    let spawn_details = find_floor_spawn(&floor_query, FloorSpawnPreference::LeftMost);

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
            damage: SLIME_ATTACK_DAMAGE,
            radius: SLIME_ATTACK_RADIUS,
            cooldown: {
                let mut timer = Timer::from_seconds(SLIME_ATTACK_COOLDOWN, TimerMode::Repeating);
                timer.set_elapsed(timer.duration());
                timer
            },
        },
    ));
}

pub fn spawn_cyclops(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile)>,
    existing_cyclops: Query<Entity, With<Cyclops>>,
) {
    if !existing_cyclops.is_empty() {
        return;
    }

    let spawn_details = find_floor_spawn(&floor_query, FloorSpawnPreference::RightMost);

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
    player_query: Query<&Transform, (With<Player>, Without<Slime>)>,
    mut slime_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
        ),
        (With<Slime>, Without<Player>),
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
    player_query: Query<&Transform, (With<Player>, Without<Cyclops>)>,
    mut cyclops_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
            &mut CyclopsCharge,
        ),
        (With<Cyclops>, Without<Player>),
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
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut attacker_query: Query<(&Transform, &mut EnemyAttack), With<Slime>>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
) {
    let mut player_iter = player_query.iter_mut();
    let Some((player_transform, mut health)) = player_iter.next() else {
        return;
    };

    let player_position = player_transform.translation.truncate();

    for (attacker_transform, mut attack) in &mut attacker_query {
        attack.cooldown.tick(time.delta());

        let distance = attacker_transform
            .translation
            .truncate()
            .distance(player_position);

        if distance <= attack.radius && attack.cooldown.finished() {
            let new_health = (health.current - attack.damage).max(0);

            if new_health != health.current {
                health.current = new_health;
                info!(
                    "史萊姆攻擊造成 {} 傷害，玩家剩餘 HP: {}",
                    attack.damage, health.current
                );

                damage_events.write(PlayerDamagedEvent {
                    damage: attack.damage,
                    remaining_health: health.current,
                });
            }

            attack.cooldown.reset();
        }
    }
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
        (With<M>, Without<Player>),
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
    floor_query: &Query<(&Transform, &RoomTile)>,
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
