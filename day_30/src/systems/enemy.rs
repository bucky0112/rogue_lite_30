use super::health::{PlayerDamagedEvent, PlayerRespawnedEvent};
use super::items::{random_pickup_effect, spawn_pickup_entity};
use crate::components::level::LevelEntity;
use crate::components::*;
use crate::constants::*;
use crate::resources::{EntranceLocation, GameSession};
use bevy::prelude::*;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};

#[derive(Event, Clone, Copy, Debug)]
pub struct EnemyDefeatedEvent {
    pub experience: u32,
    pub enemy_name: &'static str,
}

#[derive(Event, Clone, Copy)]
pub struct EnemyAttackHitEvent;

#[derive(Event, Clone, Copy, Debug)]
pub struct EnemyHitEvent {
    pub entity: Entity,
    pub position: Vec3,
    pub damage: i32,
    pub remaining_health: i32,
}

#[derive(Event, Clone, Copy)]
pub struct BossWizardSpellCastEvent;

pub fn spawn_slime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    existing_slimes: Query<Entity, With<Slime>>,
    entrance_location: Option<Res<EntranceLocation>>,
) {
    if !existing_slimes.is_empty() {
        return;
    }

    spawn_slime_internal(
        &mut commands,
        asset_server.as_ref(),
        &floor_query,
        entrance_location.as_deref(),
    );
}

pub fn spawn_cyclops(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    existing_cyclops: Query<Entity, With<Cyclops>>,
    entrance_location: Option<Res<EntranceLocation>>,
) {
    if !existing_cyclops.is_empty() {
        return;
    }

    spawn_cyclops_internal(
        &mut commands,
        asset_server.as_ref(),
        &floor_query,
        entrance_location.as_deref(),
    );
}

pub fn spawn_spider(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    floor_query: Query<(&Transform, &RoomTile), Without<Enemy>>,
    existing_spiders: Query<Entity, With<Spider>>,
    entrance_location: Option<Res<EntranceLocation>>,
) {
    if !existing_spiders.is_empty() {
        return;
    }

    spawn_spider_internal(
        &mut commands,
        asset_server.as_ref(),
        &floor_query,
        entrance_location.as_deref(),
    );
}

fn spawn_slime_internal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    floor_query: &Query<(&Transform, &RoomTile), Without<Enemy>>,
    entrance_location: Option<&EntranceLocation>,
) {
    let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
    let fallback_position = entrance_location
        .map(|entrance| {
            Vec3::new(
                entrance.position.x,
                entrance.position.y + tile_span * 2.0,
                9.0,
            )
        })
        .unwrap_or_else(|| Vec3::new(0.0, 0.0, 9.0));

    let (spawn_position, patrol_origin, patrol_range, initial_direction, used_fallback) =
        if let Some(details) = find_floor_spawn(floor_query, FloorSpawnPreference::LeftMost) {
            let corridor_width = (details.max_x - details.min_x).abs();
            if corridor_width < tile_span * 2.0 {
                (
                    fallback_position,
                    fallback_position,
                    SLIME_PATROL_RANGE,
                    1.0,
                    true,
                )
            } else {
                let values = details.into_values(FloorSpawnPreference::LeftMost);
                (values.0, values.1, values.2, values.3, false)
            }
        } else {
            (
                fallback_position,
                fallback_position,
                SLIME_PATROL_RANGE,
                1.0,
                true,
            )
        };

    if used_fallback {
        warn!(
            "Using fallback spawn position for slime, fallback_position={:?}",
            patrol_origin
        );
    } else {
        dev_info!(
            "Slime spawn floor position: {:?}, range={}",
            patrol_origin,
            patrol_range
        );
    }

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
    entrance_location: Option<&EntranceLocation>,
) {
    let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
    let fallback_position = entrance_location
        .map(|entrance| {
            Vec3::new(
                entrance.position.x + tile_span * 4.0,
                entrance.position.y + tile_span * 2.0,
                9.0,
            )
        })
        .unwrap_or_else(|| Vec3::new(tile_span * 4.0, 0.0, 9.0));

    let (spawn_position, patrol_origin, patrol_range, initial_direction, used_fallback) =
        if let Some(details) = find_floor_spawn(floor_query, FloorSpawnPreference::RightMost) {
            let corridor_width = (details.max_x - details.min_x).abs();
            if corridor_width < tile_span * 2.0 {
                (
                    fallback_position,
                    fallback_position,
                    CYCLOPS_PATROL_RANGE,
                    -1.0,
                    true,
                )
            } else {
                let values = details.into_values(FloorSpawnPreference::RightMost);
                (values.0, values.1, values.2, values.3, false)
            }
        } else {
            (
                fallback_position,
                fallback_position,
                CYCLOPS_PATROL_RANGE,
                -1.0,
                true,
            )
        };

    if used_fallback {
        warn!(
            "Cyclops fallback spawn used; fallback_position={:?}",
            patrol_origin
        );
    } else {
        dev_info!(
            "Cyclops spawn floor position: {:?}, range={}",
            patrol_origin,
            patrol_range
        );
    }

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

fn spawn_spider_internal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    floor_query: &Query<(&Transform, &RoomTile), Without<Enemy>>,
    entrance_location: Option<&EntranceLocation>,
) {
    let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
    let fallback_position = entrance_location
        .map(|entrance| {
            Vec3::new(
                entrance.position.x - tile_span * 2.0,
                entrance.position.y + tile_span * 2.0,
                9.0,
            )
        })
        .unwrap_or_else(|| Vec3::new(-tile_span * 2.0, tile_span * 2.0, 9.0));

    let (spawn_position, patrol_origin, patrol_range, initial_direction, used_fallback) =
        if let Some(details) = find_floor_spawn(floor_query, FloorSpawnPreference::Center) {
            let corridor_width = (details.max_x - details.min_x).abs();
            if corridor_width < tile_span * 2.5 {
                (
                    fallback_position,
                    fallback_position,
                    SPIDER_PATROL_RANGE,
                    1.0,
                    true,
                )
            } else {
                let values = details.into_values(FloorSpawnPreference::Center);
                (values.0, values.1, values.2, values.3, false)
            }
        } else {
            (
                fallback_position,
                fallback_position,
                SPIDER_PATROL_RANGE,
                1.0,
                true,
            )
        };

    if used_fallback {
        warn!(
            "Using fallback spawn position for spider, fallback_position={:?}",
            patrol_origin
        );
    } else {
        dev_info!(
            "Spider spawn floor position: {:?}, range={}",
            patrol_origin,
            patrol_range
        );
    }

    let floor_tiles: HashSet<(i32, i32)> = floor_query
        .iter()
        .filter(|(_, tile)| {
            matches!(
                tile.tile_type,
                RoomTileType::Floor | RoomTileType::FloorOutdoor
            )
        })
        .map(|(transform, _)| {
            (
                (transform.translation.x / tile_span).round() as i32,
                (transform.translation.y / tile_span).round() as i32,
            )
        })
        .collect();

    let (adjusted_origin, adjusted_range) =
        resolve_spider_patrol_bounds(spawn_position, tile_span, &floor_tiles);

    commands.spawn((
        Enemy,
        Spider,
        Sprite::from_image(asset_server.load("characters/enemies/spider.png")),
        Transform::from_translation(adjusted_origin).with_scale(Vec3::splat(SPIDER_SCALE)),
        Health::new(SPIDER_HEALTH),
        Attack::new(SPIDER_BASE_ATTACK),
        Defense::new(SPIDER_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: adjusted_origin,
            range: adjusted_range,
            direction: initial_direction,
        },
        EnemyAlert {
            trigger_radius: 0.0,
            leash_radius: 0.0,
        },
        EnemySpeeds {
            patrol: SPIDER_PATROL_SPEED,
            chase: SPIDER_PATROL_SPEED,
        },
        EnemyAttack {
            radius: SPIDER_ATTACK_RADIUS,
            cooldown: {
                let mut timer = Timer::from_seconds(SPIDER_ATTACK_COOLDOWN, TimerMode::Repeating);
                timer.set_elapsed(timer.duration());
                timer
            },
        },
    ));
}

pub fn slime_ai_system(
    time: Res<Time>,
    session: Res<GameSession>,
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
    if !session.is_playing() {
        return;
    }

    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    process_enemy_ai::<Slime>(time.as_ref(), player_position, &mut slime_query);
}

pub fn spider_ai_system(
    time: Res<Time>,
    session: Res<GameSession>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerDead>)>,
    mut spider_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
        ),
        (With<Spider>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    if !session.is_playing() {
        return;
    }

    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    let delta_secs = time.delta_secs();

    // Spiders keep pacing their patrol lane and nudge away from the player when pressed.
    for (mut transform, mut ai_state, mut patrol, _alert, speeds) in &mut spider_query {
        ai_state.state = EnemyBehaviorState::Patrolling;

        let (min_x, max_x) = patrol.bounds();
        if patrol.direction.abs() < f32::EPSILON {
            patrol.direction = 1.0;
        }

        let mut direction = patrol.direction;

        if let Some(player_pos) = player_position {
            let to_player = player_pos - transform.translation;
            let planar = to_player.truncate();
            let distance = planar.length();

            if distance < SPIDER_MIN_ATTACK_DISTANCE {
                let desired = if planar.x.abs() > f32::EPSILON {
                    -planar.x.signum()
                } else {
                    -direction.signum()
                };

                let step = speeds.patrol * 1.0 * delta_secs;
                let projected = transform.translation.x + desired * step;

                direction = if desired < 0.0 && projected < min_x {
                    1.0
                } else if desired > 0.0 && projected > max_x {
                    -1.0
                } else if desired.abs() < f32::EPSILON {
                    direction
                } else {
                    desired
                };
            }
        }

        let mut translation = transform.translation;
        translation.x += direction * speeds.patrol * delta_secs;

        if translation.x > max_x {
            translation.x = max_x;
            direction = -1.0;
        } else if translation.x < min_x {
            translation.x = min_x;
            direction = 1.0;
        }

        translation.y = patrol.origin.y;
        translation.z = patrol.origin.z;
        transform.translation = translation;
        patrol.direction = direction;
    }
}

pub fn spider_ranged_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    mut spider_query: Query<(&Transform, &mut EnemyAttack, &Attack), With<Spider>>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerDead>)>,
) {
    if !session.is_playing() {
        return;
    }

    let Some(player_transform) = player_query.iter().next() else {
        return;
    };

    let player_position = player_transform.translation.truncate();

    for (transform, mut attack, attack_stat) in &mut spider_query {
        attack.cooldown.tick(time.delta());

        if !attack.cooldown.finished() {
            continue;
        }

        let to_player = player_position - transform.translation.truncate();
        let distance = to_player.length();

        if distance > attack.radius || distance < SPIDER_MIN_ATTACK_DISTANCE {
            continue;
        }

        let direction = to_player.normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }

        spawn_spider_web_projectile(
            &mut commands,
            transform.translation,
            direction,
            attack_stat.value(),
        );

        attack.cooldown.reset();
    }
}

pub fn cyclops_ai_system(
    time: Res<Time>,
    session: Res<GameSession>,
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
    if !session.is_playing() {
        return;
    }

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

pub fn boss_wizard_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    mut spell_events: EventWriter<BossWizardSpellCastEvent>,
    player_query: Query<&Transform, (With<Player>, Without<BossWizard>, Without<PlayerDead>)>,
    mut wizard_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
            &mut EnemyAttack,
            &Attack,
        ),
        (With<BossWizard>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    if !session.is_playing() {
        return;
    }

    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    let delta = time.delta();
    let delta_secs = time.delta_secs();

    for (mut transform, mut ai_state, mut patrol, alert, speeds, mut attack, attack_stat) in
        &mut wizard_query
    {
        attack.cooldown.tick(delta);
        let (min_x, max_x) = patrol.bounds();

        if let Some(player_pos) = player_position {
            let to_player = player_pos - transform.translation;
            let distance = to_player.truncate().length();

            if distance <= alert.trigger_radius {
                ai_state.state = EnemyBehaviorState::Chasing;

                let direction = to_player.truncate().normalize_or_zero();
                let dx = player_pos.x - transform.translation.x;

                if distance < WIZARD_BOSS_CAST_MIN_DISTANCE {
                    let retreat_dir = if dx.abs() > f32::EPSILON {
                        -dx.signum()
                    } else {
                        -patrol.direction
                    };
                    let retreat = retreat_dir * speeds.chase * delta_secs;
                    transform.translation.x =
                        (transform.translation.x + retreat).clamp(min_x, max_x);
                } else if distance > WIZARD_BOSS_ATTACK_RADIUS {
                    let advance_dir = if dx.abs() > f32::EPSILON {
                        dx.signum()
                    } else {
                        patrol.direction
                    };
                    let advance = advance_dir * speeds.patrol * delta_secs;
                    transform.translation.x =
                        (transform.translation.x + advance).clamp(min_x, max_x);
                } else {
                    transform.translation.x = transform.translation.x.clamp(min_x, max_x);
                }

                transform.translation.y = patrol.origin.y;
                transform.translation.z = patrol.origin.z;

                if distance <= WIZARD_BOSS_ATTACK_RADIUS && attack.cooldown.finished() {
                    spawn_wizard_projectile(
                        &mut commands,
                        transform.translation,
                        direction,
                        attack_stat.value(),
                    );
                    spell_events.write(BossWizardSpellCastEvent);
                    attack.cooldown.reset();
                }

                if dx.abs() > f32::EPSILON {
                    patrol.direction = dx.signum();
                }

                continue;
            }
        }

        ai_state.state = EnemyBehaviorState::Patrolling;

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

pub fn boss_wizard_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    mut projectile_query: Query<(
        Entity,
        &mut Transform,
        &BossWizardProjectile,
        &mut BossWizardProjectileLifetime,
    )>,
    player_target_query: Query<
        (&Transform, Option<&Defense>),
        (
            With<Player>,
            Without<PlayerDead>,
            Without<BossWizardProjectile>,
        ),
    >,
    mut player_health_query: Query<
        &mut Health,
        (
            With<Player>,
            Without<PlayerDead>,
            Without<BossWizardProjectile>,
        ),
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut enemy_attack_events: EventWriter<EnemyAttackHitEvent>,
) {
    if !session.is_playing() {
        return;
    }

    let delta = time.delta();
    let delta_secs = time.delta_secs();

    let player_target = player_target_query.single().ok();
    let player_position = player_target.map(|(transform, _)| transform.translation.truncate());
    let defense_value = player_target.and_then(|(_, defense)| defense.map(|value| value.value()));

    let mut hits: Vec<(Entity, i32)> = Vec::new();

    for (entity, mut transform, projectile, mut lifetime) in &mut projectile_query {
        transform.translation.x += projectile.velocity.x * delta_secs;
        transform.translation.y += projectile.velocity.y * delta_secs;

        lifetime.timer.tick(delta);
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(player_pos) = player_position {
            let distance = transform.translation.truncate().distance(player_pos);
            if distance <= WIZARD_BOSS_PROJECTILE_HIT_RADIUS {
                let damage = compute_damage(projectile.damage, defense_value);
                hits.push((entity, damage));
            }
        }
    }

    if hits.is_empty() {
        return;
    }

    hits.sort_unstable_by_key(|(entity, _)| entity.index());

    if let Ok(mut health) = player_health_query.single_mut() {
        for (entity, damage) in hits {
            commands.entity(entity).despawn();

            if damage <= 0 {
                continue;
            }

            let new_health = (health.current - damage).max(0);
            if new_health != health.current {
                health.current = new_health;
                damage_events.write(PlayerDamagedEvent {
                    damage,
                    remaining_health: health.current,
                });
                enemy_attack_events.write(EnemyAttackHitEvent);
            }
        }
    } else {
        for (entity, _) in hits {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spider_web_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    mut projectile_query: Query<
        (
            Entity,
            &mut Transform,
            &SpiderWebProjectile,
            &mut SpiderWebProjectileLifetime,
        ),
        With<SpiderWebProjectile>,
    >,
    wall_query: Query<(Entity, &RoomTile, &Transform), Without<SpiderWebProjectile>>,
    door_query: Query<&Door>,
    mut player_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&Defense>,
            Option<&mut Poisoned>,
        ),
        (
            With<Player>,
            Without<PlayerDead>,
            Without<SpiderWebProjectile>,
        ),
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut enemy_attack_events: EventWriter<EnemyAttackHitEvent>,
) {
    if !session.is_playing() {
        return;
    }

    let mut player_iter = player_query.iter_mut();
    let Some((player_entity, player_transform, mut health, defense, poison_option)) =
        player_iter.next()
    else {
        return;
    };

    let player_position = player_transform.translation.truncate();
    let defense_value = defense.map(|value| value.value());
    let mut poison_component = poison_option;
    let mut poison_active = poison_component.is_some();

    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let mut blocking_tiles: HashMap<(i32, i32), (RoomTileType, Entity)> = HashMap::new();

    for (entity, room_tile, transform) in &wall_query {
        let tile_key = (
            (transform.translation.x / tile_size).round() as i32,
            (transform.translation.y / tile_size).round() as i32,
        );

        let new_value = (room_tile.tile_type, entity);
        match blocking_tiles.get(&tile_key) {
            Some((existing_type, existing_entity)) => {
                let existing_priority = projectile_tile_priority(*existing_type);
                let new_priority = projectile_tile_priority(room_tile.tile_type);
                if new_priority > existing_priority
                    || (new_priority == existing_priority
                        && entity.index() < existing_entity.index())
                {
                    blocking_tiles.insert(tile_key, new_value);
                }
            }
            None => {
                blocking_tiles.insert(tile_key, new_value);
            }
        }
    }

    for (entity, mut transform, projectile, mut lifetime) in &mut projectile_query {
        let previous_translation = transform.translation;

        let delta_secs = time.delta_secs();
        transform.translation.x += projectile.velocity.x * delta_secs;
        transform.translation.y += projectile.velocity.y * delta_secs;

        let angle = projectile.direction.y.atan2(projectile.direction.x);
        transform.rotation = Quat::from_rotation_z(angle);

        lifetime.timer.tick(time.delta());

        let previous_center = previous_translation.truncate();
        let center = transform.translation.truncate();
        let to_player = player_position - center;
        let along = to_player.dot(projectile.direction);
        let half_length = SPIDER_WEB_PROJECTILE_LENGTH * 0.5;

        let mut blocked = false;

        let travel = center - previous_center;
        let steps = 4;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let sample = previous_center + travel * t;
            if projectile_blocked_at(sample, tile_size, &blocking_tiles, &door_query) {
                blocked = true;
                break;
            }
        }

        if !blocked {
            let forward_tip = center + projectile.direction * half_length;
            let backward_tip = center - projectile.direction * half_length;

            if projectile_blocked_at(forward_tip, tile_size, &blocking_tiles, &door_query)
                || projectile_blocked_at(backward_tip, tile_size, &blocking_tiles, &door_query)
            {
                blocked = true;
            }
        }

        if blocked {
            commands.entity(entity).despawn();
            continue;
        }

        if along.abs() <= half_length {
            let perpendicular = (to_player - projectile.direction * along).length();
            if perpendicular <= SPIDER_WEB_PROJECTILE_HIT_RADIUS {
                let damage = compute_damage(projectile.damage, defense_value);
                if damage > 0 {
                    let new_health = (health.current - damage).max(0);
                    if new_health != health.current {
                        health.current = new_health;
                        damage_events.write(PlayerDamagedEvent {
                            damage,
                            remaining_health: health.current,
                        });
                    }
                }

                if let Some(poison) = poison_component.as_mut() {
                    poison.reset_timer();
                } else if !poison_active {
                    commands.entity(player_entity).insert(Poisoned::new(
                        PLAYER_POISON_TICK_SECONDS,
                        PLAYER_POISON_TICK_DAMAGE,
                    ));
                    poison_active = true;
                }

                dev_info!(
                    "Spider web struck player for {} damage (poison active={})",
                    damage,
                    poison_active
                );

                enemy_attack_events.write(EnemyAttackHitEvent);
                commands.entity(entity).despawn();
                continue;
            }
        }

        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_wizard_projectile(commands: &mut Commands, origin: Vec3, direction: Vec2, damage: i32) {
    let direction = direction.normalize_or_zero();
    if direction == Vec2::ZERO {
        return;
    }

    let offset_distance = WIZARD_BOSS_STAFF_OFFSET_X.max(6.0) + WIZARD_BOSS_PROJECTILE_SIZE * 0.5;
    let spawn_position = Vec3::new(
        origin.x + direction.x * offset_distance,
        origin.y + direction.y * offset_distance,
        origin.z + WIZARD_BOSS_CAST_HEIGHT_OFFSET,
    );

    commands.spawn((
        LevelEntity,
        BossWizardProjectile {
            velocity: direction * WIZARD_BOSS_PROJECTILE_SPEED,
            damage,
        },
        BossWizardProjectileLifetime {
            timer: Timer::from_seconds(WIZARD_BOSS_PROJECTILE_LIFETIME, TimerMode::Once),
        },
        Sprite::from_color(
            WIZARD_BOSS_PROJECTILE_COLOR,
            Vec2::splat(WIZARD_BOSS_PROJECTILE_SIZE),
        ),
        Transform::from_translation(spawn_position),
        Name::new("BossWizardProjectile"),
    ));
}

fn spawn_spider_web_projectile(
    commands: &mut Commands,
    origin: Vec3,
    direction: Vec2,
    damage: i32,
) {
    let direction = direction.normalize_or_zero();
    if direction == Vec2::ZERO {
        return;
    }

    let forward_offset = SPIDER_WEB_PROJECTILE_SPAWN_OFFSET + SPIDER_WEB_PROJECTILE_LENGTH * 0.5;

    let spawn_position = Vec3::new(
        origin.x + direction.x * forward_offset,
        origin.y + direction.y * forward_offset,
        origin.z + 12.0,
    );

    let angle = direction.y.atan2(direction.x);

    let mut transform = Transform::from_translation(spawn_position);
    transform.rotation = Quat::from_rotation_z(angle);

    commands.spawn((
        LevelEntity,
        SpiderWebProjectile {
            velocity: direction * SPIDER_WEB_PROJECTILE_SPEED,
            damage,
            direction,
        },
        SpiderWebProjectileLifetime {
            timer: Timer::from_seconds(SPIDER_WEB_PROJECTILE_LIFETIME, TimerMode::Once),
        },
        Sprite {
            color: Color::srgba(0.95, 0.98, 1.0, 0.88),
            custom_size: Some(Vec2::new(
                SPIDER_WEB_PROJECTILE_LENGTH,
                SPIDER_WEB_PROJECTILE_THICKNESS,
            )),
            ..Default::default()
        },
        transform,
        Name::new("SpiderWebProjectile"),
    ));
}

fn projectile_blocked_at(
    point: Vec2,
    tile_size: f32,
    blocking_tiles: &HashMap<(i32, i32), (RoomTileType, Entity)>,
    door_query: &Query<&Door>,
) -> bool {
    let tile_key = (
        (point.x / tile_size).round() as i32,
        (point.y / tile_size).round() as i32,
    );

    if let Some((tile_type, entity)) = blocking_tiles.get(&tile_key) {
        tile_blocks_projectile(*tile_type, *entity, door_query)
    } else {
        false
    }
}

fn tile_blocks_projectile(
    tile_type: RoomTileType,
    entity: Entity,
    door_query: &Query<&Door>,
) -> bool {
    match tile_type {
        RoomTileType::Floor | RoomTileType::FloorOutdoor | RoomTileType::DoorOpen => false,
        RoomTileType::DoorClosed => door_query
            .get(entity)
            .map(|door| !door.is_open)
            .unwrap_or(true),
        RoomTileType::WallNInnerCornerW
        | RoomTileType::WallNInnerMid
        | RoomTileType::WallNInnerCornerE
        | RoomTileType::WallSInnerCapL
        | RoomTileType::WallSInnerMid
        | RoomTileType::WallSInnerCapR
        | RoomTileType::WallSOuterCapL
        | RoomTileType::WallSOuterMid
        | RoomTileType::WallSOuterCapR
        | RoomTileType::WallESide
        | RoomTileType::WallWSide => true,
    }
}

fn projectile_tile_priority(tile_type: RoomTileType) -> u8 {
    match tile_type {
        RoomTileType::Floor | RoomTileType::FloorOutdoor => 0,
        RoomTileType::DoorOpen => 1,
        RoomTileType::DoorClosed => 2,
        RoomTileType::WallNInnerCornerW
        | RoomTileType::WallNInnerMid
        | RoomTileType::WallNInnerCornerE
        | RoomTileType::WallSInnerCapL
        | RoomTileType::WallSInnerMid
        | RoomTileType::WallSInnerCapR
        | RoomTileType::WallSOuterCapL
        | RoomTileType::WallSOuterMid
        | RoomTileType::WallSOuterCapR
        | RoomTileType::WallESide
        | RoomTileType::WallWSide => 3,
    }
}

fn resolve_spider_patrol_bounds(
    position: Vec3,
    tile_size: f32,
    floor_tiles: &HashSet<(i32, i32)>,
) -> (Vec3, f32) {
    let tile_x = (position.x / tile_size).round() as i32;
    let tile_y = (position.y / tile_size).round() as i32;

    let mut left_tile = tile_x;
    let mut left_center = position.x;
    while floor_tiles.contains(&(left_tile - 1, tile_y)) {
        left_tile -= 1;
        left_center -= tile_size;
    }

    let mut right_tile = tile_x;
    let mut right_center = position.x;
    while floor_tiles.contains(&(right_tile + 1, tile_y)) {
        right_tile += 1;
        right_center += tile_size;
    }

    let mut min_edge = left_center - tile_size * 0.5;
    let mut max_edge = right_center + tile_size * 0.5;
    let margin = tile_size * 0.25;

    min_edge += margin;
    max_edge -= margin;

    if max_edge <= min_edge {
        let origin = Vec3::new(position.x, position.y, position.z);
        return (origin, tile_size * 0.75);
    }

    let width = max_edge - min_edge;
    if width <= tile_size * 0.3 {
        let origin_x = position.x;
        return (
            Vec3::new(origin_x, position.y, position.z),
            tile_size * 0.75,
        );
    }

    let origin_x = (min_edge + max_edge) * 0.5;
    let range = (width * 0.5)
        .min(SPIDER_PATROL_RANGE)
        .clamp(tile_size * 0.75, SPIDER_PATROL_RANGE);

    (Vec3::new(origin_x, position.y, position.z), range)
}

pub fn mimic_ai_system(
    time: Res<Time>,
    session: Res<GameSession>,
    player_query: Query<&Transform, (With<Player>, Without<Mimic>, Without<PlayerDead>)>,
    mut mimic_query: Query<
        (
            &mut Transform,
            &mut EnemyAIState,
            &mut EnemyPatrol,
            &EnemyAlert,
            &EnemySpeeds,
        ),
        (With<Mimic>, Without<Player>, Without<EnemyDeathEffect>),
    >,
) {
    if !session.is_playing() {
        return;
    }

    let player_position = player_query
        .iter()
        .next()
        .map(|transform| transform.translation);

    process_enemy_ai::<Mimic>(time.as_ref(), player_position, &mut mimic_query);
}

pub fn enemy_contact_attack_system(
    time: Res<Time>,
    session: Res<GameSession>,
    mut player_query: Query<
        (&Transform, &mut Health, Option<&Defense>),
        (With<Player>, Without<PlayerDead>),
    >,
    mut attacker_query: Query<
        (
            &Transform,
            &mut EnemyAttack,
            &Attack,
            Option<&Slime>,
            Option<&Mimic>,
            Option<&Cyclops>,
        ),
        (
            With<Enemy>,
            Without<EnemyDeathEffect>,
            Without<BossWizard>,
            Without<Spider>,
        ),
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut enemy_attack_events: EventWriter<EnemyAttackHitEvent>,
) {
    if !session.is_playing() {
        return;
    }

    let mut player_iter = player_query.iter_mut();
    let Some((player_transform, mut health, defense)) = player_iter.next() else {
        return;
    };

    let player_position = player_transform.translation.truncate();

    let defense_value = defense.map(|value| value.value());

    for (attacker_transform, mut attack, attack_stat, is_slime, is_mimic, is_cyclops) in
        &mut attacker_query
    {
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
                let attacker_name = if is_mimic.is_some() {
                    "Mimic"
                } else if is_slime.is_some() {
                    "Slime"
                } else if is_cyclops.is_some() {
                    "Cyclops"
                } else {
                    "Enemy"
                };
                dev_info!(
                    "{} attack dealt {} damage; player HP now {}",
                    attacker_name,
                    damage,
                    health.current
                );

                damage_events.write(PlayerDamagedEvent {
                    damage,
                    remaining_health: health.current,
                });
                enemy_attack_events.write(EnemyAttackHitEvent);
            }

            attack.cooldown.reset();
        }
    }
}

pub fn despawn_dead_enemies_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut defeated_events: EventWriter<EnemyDefeatedEvent>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Health,
            Option<&EnemyDeathEffect>,
            &mut Sprite,
            Option<&Slime>,
            Option<&Cyclops>,
            Option<&Spider>,
            Option<&Mimic>,
            Option<&BossWizard>,
        ),
        With<Enemy>,
    >,
) {
    let mut rng = thread_rng();

    for (
        entity,
        transform,
        health,
        death_effect,
        mut sprite,
        slime,
        cyclops,
        spider,
        mimic,
        wizard,
    ) in &mut query
    {
        if health.current > 0 {
            continue;
        }

        if death_effect.is_some() {
            continue;
        }

        sprite.color.set_alpha(1.0);

        let (experience_reward, enemy_label) = if wizard.is_some() {
            (WIZARD_BOSS_EXPERIENCE_REWARD, "Wizard Boss")
        } else if slime.is_some() {
            (SLIME_EXPERIENCE_REWARD, "Slime")
        } else if cyclops.is_some() {
            (CYCLOPS_EXPERIENCE_REWARD, "Cyclops")
        } else if spider.is_some() {
            (SPIDER_EXPERIENCE_REWARD, "Spider")
        } else if mimic.is_some() {
            (MIMIC_EXPERIENCE_REWARD, "Mimic")
        } else {
            (0, "Enemy")
        };

        if experience_reward > 0 {
            defeated_events.write(EnemyDefeatedEvent {
                experience: experience_reward,
                enemy_name: enemy_label,
            });

            dev_info!(
                "Defeated {} and earned {} EXP",
                enemy_label,
                experience_reward
            );
        }

        let drop_effect = if spider.is_some() {
            PickupEffect::CurePoison
        } else {
            random_pickup_effect(&mut rng)
        };
        let drop_label = match &drop_effect {
            PickupEffect::Heal(_) => "Red potion",
            PickupEffect::RestoreStamina(_) => "Green potion",
            PickupEffect::CurePoison => "Antidote",
            PickupEffect::EquipShield(kind) => kind.display_name(),
            PickupEffect::EquipWeapon(kind) => kind.display_name(),
        };

        spawn_pickup_entity(
            &mut commands,
            asset_server.as_ref(),
            drop_effect,
            transform.translation,
        );

        commands.entity(entity).insert(EnemyDeathEffect {
            timer: Timer::from_seconds(ENEMY_DEATH_FADE_SECONDS, TimerMode::Once),
        });

        dev_info!("Enemy defeated, dropped {} and started fading", drop_label);
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
            dev_info!("Enemy fade-out finished; entity removed");
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
    spider_entities: Query<Entity, With<Spider>>,
    projectiles: Query<Entity, With<SpiderWebProjectile>>,
    entrance_location: Option<Res<EntranceLocation>>,
) {
    let mut triggered = false;
    for _ in respawn_events.read() {
        triggered = true;
    }

    if !triggered {
        return;
    }

    for entity in &projectiles {
        commands.entity(entity).despawn();
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
        spawn_slime_internal(
            &mut commands,
            asset_server.as_ref(),
            &floor_query,
            entrance_location.as_deref(),
        );
    }

    if cyclops_entities.iter().next().is_none() {
        spawn_cyclops_internal(
            &mut commands,
            asset_server.as_ref(),
            &floor_query,
            entrance_location.as_deref(),
        );
    }

    if spider_entities.iter().next().is_none() {
        spawn_spider_internal(
            &mut commands,
            asset_server.as_ref(),
            &floor_query,
            entrance_location.as_deref(),
        );
    }

    dev_info!("Enemy roster reset to initial state");
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
    Center,
}

struct FloorSpawnInfo {
    position: Vec3,
    min_x: f32,
    max_x: f32,
}

impl FloorSpawnInfo {
    fn into_values(self, preference: FloorSpawnPreference) -> (Vec3, Vec3, f32, f32) {
        let tile_span = ROOM_TILE_SIZE * PLAYER_SCALE;
        let patrol_center_x = (self.min_x + self.max_x) * 0.5;
        let spawn_position = match preference {
            FloorSpawnPreference::LeftMost => self.position,
            FloorSpawnPreference::RightMost => self.position,
            FloorSpawnPreference::Center => {
                Vec3::new(patrol_center_x, self.position.y, self.position.z)
            }
        };

        let max_range = match preference {
            FloorSpawnPreference::LeftMost => SLIME_PATROL_RANGE,
            FloorSpawnPreference::RightMost => CYCLOPS_PATROL_RANGE,
            FloorSpawnPreference::Center => SPIDER_PATROL_RANGE,
        };
        let half_width = (self.max_x - self.min_x) * 0.5;
        let margin = tile_span * 0.1;
        let patrol_range = (half_width - margin).max(tile_span * 0.25).min(max_range);
        let direction = match preference {
            FloorSpawnPreference::LeftMost => 1.0,
            FloorSpawnPreference::RightMost => -1.0,
            FloorSpawnPreference::Center => 1.0,
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
    let mut best: Option<FloorSpawnInfo> = None;

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

        let candidate = FloorSpawnInfo {
            position: Vec3::new(transform.translation.x, transform.translation.y, 9.0),
            min_x: entry.0,
            max_x: entry.1,
        };

        let keep_current = if let Some(current) = &best {
            match preference {
                FloorSpawnPreference::LeftMost => current.position.x <= candidate.position.x,
                FloorSpawnPreference::RightMost => current.position.x >= candidate.position.x,
                FloorSpawnPreference::Center => {
                    let current_center = (current.min_x + current.max_x) * 0.5;
                    let candidate_center = (candidate.min_x + candidate.max_x) * 0.5;
                    let current_distance = current_center.abs();
                    let candidate_distance = candidate_center.abs();

                    if candidate_distance + tile_span * 0.1 < current_distance {
                        false
                    } else if candidate_distance > current_distance + tile_span * 0.1 {
                        true
                    } else {
                        let current_width = (current.max_x - current.min_x).abs();
                        let candidate_width = (candidate.max_x - candidate.min_x).abs();
                        current_width >= candidate_width
                    }
                }
            }
        } else {
            false
        };

        if keep_current {
            continue;
        }

        best = Some(candidate);
    }

    best
}
