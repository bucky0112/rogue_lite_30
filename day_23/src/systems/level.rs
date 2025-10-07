use std::collections::HashSet;

use crate::components::{
    Attack, Cyclops, CyclopsCharge, Defense, Enemy, EnemyAIState, EnemyAlert, EnemyAttack,
    EnemyBehaviorState, EnemyPatrol, EnemySpeeds, Health, Slime,
    level::{LevelEntity, LevelExitDoor},
    player::{InputVector, Player, PlayerDead, Velocity},
    world::{Door, EnvironmentProp, RoomTile, RoomTileType},
};
use crate::constants::*;
use crate::resources::{
    EntranceLocation, EnvironmentAssets, LevelBuildContext, LevelDefinition, LevelExitAssets,
    LevelState, RoomAssets,
};
use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

#[derive(Event, Debug, Clone, Copy)]
pub struct LevelAdvanceRequestEvent {
    pub target_level: usize,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct LevelLoadedEvent {
    pub index: usize,
    pub name: &'static str,
}

pub fn schedule_initial_level(mut build_context: ResMut<LevelBuildContext>) {
    if build_context.pending_layout.is_none() && build_context.pending_finalize.is_none() {
        build_context.pending_layout = Some(0);
    }
}

pub fn handle_level_requests(
    mut events: EventReader<LevelAdvanceRequestEvent>,
    mut level_state: ResMut<LevelState>,
    mut build_context: ResMut<LevelBuildContext>,
) {
    if build_context.pending_layout.is_some() || build_context.pending_finalize.is_some() {
        events.clear();
        return;
    }

    for event in events.read() {
        let Some(expected_next) = level_state.next_index() else {
            info!("📦 已在最後一關，無法再前往更高關卡");
            continue;
        };

        if event.target_level != expected_next {
            warn!(
                "忽略無效關卡請求：target={}, expected={}",
                event.target_level, expected_next
            );
            continue;
        }

        info!("🌀 準備載入關卡 {}", event.target_level + 1);
        level_state.set_current_index(event.target_level);
        build_context.pending_layout = Some(event.target_level);
        break;
    }
}

pub fn process_level_layout(
    mut commands: Commands,
    mut build_context: ResMut<LevelBuildContext>,
    level_state: Res<LevelState>,
    level_entities: Query<Entity, With<LevelEntity>>,
    room_assets: Res<RoomAssets>,
) {
    let Some(index) = build_context.pending_layout.take() else {
        return;
    };

    clear_level_entities(&mut commands, &level_entities);

    let definition = level_state.definition(index).clone();
    spawn_layout_for_level(&mut commands, &room_assets, &definition);

    build_context.pending_finalize = Some(index);
}

pub fn finalize_level_load(
    mut commands: Commands,
    mut build_context: ResMut<LevelBuildContext>,
    level_state: Res<LevelState>,
    environment_assets: Res<EnvironmentAssets>,
    level_exit_assets: Option<Res<LevelExitAssets>>,
    asset_server: Res<AssetServer>,
    door_query: Query<&Transform, (With<Door>, With<LevelEntity>)>,
    tile_query: Query<(&Transform, &RoomTile), With<LevelEntity>>,
    mut player_query: Query<
        (
            &mut Transform,
            Option<&mut Velocity>,
            Option<&mut InputVector>,
        ),
        (With<Player>, Without<PlayerDead>, Without<LevelEntity>),
    >,
    mut level_loaded_events: EventWriter<LevelLoadedEvent>,
) {
    let Some(index) = build_context.pending_finalize.take() else {
        return;
    };

    let definition = level_state.definition(index);
    let mut rng = StdRng::seed_from_u64(definition.seed);
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;

    let Some(door_transform) = door_query.iter().min_by(|a, b| {
        a.translation
            .y
            .partial_cmp(&b.translation.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) else {
        warn!("找不到房間入口門，略過關卡初始化");
        return;
    };

    let door_position = door_transform.translation;
    let spawn_position = Vec3::new(door_position.x, door_position.y - tile_size * 2.0, 10.0);
    commands.insert_resource(EntranceLocation::new(spawn_position));

    if let Ok((mut player_transform, velocity, input_vector)) = player_query.single_mut() {
        player_transform.translation.x = spawn_position.x;
        player_transform.translation.y = spawn_position.y;
        player_transform.translation.z = spawn_position.z;

        if let Some(mut velocity) = velocity {
            *velocity = Velocity::zero();
        }
        if let Some(mut input_vector) = input_vector {
            input_vector.0 = Vec2::ZERO;
        }
    }

    let tile_samples: Vec<(Vec3, RoomTileType)> = tile_query
        .iter()
        .map(|(transform, tile)| (transform.translation, tile.tile_type))
        .collect();

    let mut floor_positions: Vec<Vec3> = tile_samples
        .iter()
        .filter_map(|(position, tile)| match tile {
            RoomTileType::Floor | RoomTileType::FloorOutdoor => Some(*position),
            _ => None,
        })
        .collect();

    floor_positions.shuffle(&mut rng);

    let portal_anchor = compute_portal_anchor(&tile_samples, tile_size).unwrap_or(Vec3::new(
        door_position.x + tile_size * 2.0,
        spawn_position.y,
        spawn_position.z,
    ));

    let exit_position = level_state
        .next_index()
        .and_then(|target| {
            level_exit_assets.as_ref().map(|assets| {
                spawn_level_exit_portal(
                    &mut commands,
                    assets.as_ref(),
                    portal_anchor,
                    tile_size,
                    target,
                )
            })
        })
        .flatten();

    spawn_environment_props_for_level(
        &mut commands,
        &environment_assets,
        definition,
        &mut rng,
        tile_size,
        &mut floor_positions,
        door_position,
        spawn_position,
        exit_position,
    );

    spawn_enemies_for_level(
        &mut commands,
        &asset_server,
        definition,
        &mut rng,
        tile_size,
        &mut floor_positions,
        door_position,
        spawn_position,
        exit_position,
    );

    info!(
        "🌌 關卡{} - {} 準備完成 (敵人數={}, 道具數={})",
        definition.index + 1,
        definition.name,
        definition.enemy_total(),
        definition.prop_plan.total()
    );

    level_loaded_events.write(LevelLoadedEvent {
        index: definition.index,
        name: definition.name,
    });
}

fn clear_level_entities(commands: &mut Commands, entities: &Query<Entity, With<LevelEntity>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_layout_for_level(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    definition: &LevelDefinition,
) {
    match &definition.layout {
        crate::resources::level::RoomLayout::Rectangle { width, height } => {
            let start_x = -(*width as i32) / 2;
            let start_y = -(*height as i32) / 2;
            crate::systems::world::generate_room_tiles(
                commands,
                room_assets,
                *width,
                *height,
                start_x,
                start_y,
                true,
            );
        }
        crate::resources::level::RoomLayout::Compound {
            room_type,
            rectangles,
        } => {
            let compound = crate::components::world::CompoundRoom {
                rectangles: rectangles.clone(),
                room_type: room_type.clone(),
            };
            crate::systems::world::spawn_compound_room(commands, room_assets, compound);
        }
    }
}

fn spawn_level_exit_portal(
    commands: &mut Commands,
    assets: &LevelExitAssets,
    anchor: Vec3,
    tile_size: f32,
    target_level: usize,
) -> Option<Vec3> {
    let position = Vec3::new(anchor.x, anchor.y + tile_size * 0.5, 11.0);

    let portal = commands
        .spawn((
            LevelEntity,
            LevelExitDoor::new(target_level),
            Transform::from_translation(position),
            GlobalTransform::default(),
            Name::new(format!("LevelExitPortal{}", target_level + 1)),
        ))
        .id();

    info!(
        "🚪 生成下一關入口：target={} @ ({:.1}, {:.1})",
        target_level + 1,
        position.x,
        position.y
    );

    let panel_offset = tile_size * 0.5;
    let scale = Vec3::splat(PLAYER_SCALE);

    commands.entity(portal).with_children(|parent| {
        parent.spawn((
            Sprite::from_image(assets.left_panel.clone()),
            Transform::from_translation(Vec3::new(-panel_offset, 0.0, 0.0)).with_scale(scale),
            Name::new("LevelExitPanelLeft"),
        ));
        parent.spawn((
            Sprite::from_image(assets.right_panel.clone()),
            Transform::from_translation(Vec3::new(panel_offset, 0.0, 0.0)).with_scale(scale),
            Name::new("LevelExitPanelRight"),
        ));
    });

    Some(position)
}

fn compute_portal_anchor(tiles: &[(Vec3, RoomTileType)], tile_size: f32) -> Option<Vec3> {
    let mut top_y = f32::NEG_INFINITY;
    let mut x_sum = 0.0;
    let mut count = 0;
    let tolerance = tile_size * 0.1;

    for (position, tile) in tiles {
        if *tile != RoomTileType::Floor {
            continue;
        }

        let y = position.y;
        if y > top_y + tolerance {
            top_y = y;
            x_sum = position.x;
            count = 1;
        } else if (y - top_y).abs() <= tolerance {
            x_sum += position.x;
            count += 1;
        }
    }

    if count == 0 {
        None
    } else {
        let x_center = x_sum / count as f32;
        Some(Vec3::new(x_center, top_y, 11.0))
    }
}

fn spawn_environment_props_for_level(
    commands: &mut Commands,
    assets: &EnvironmentAssets,
    definition: &LevelDefinition,
    rng: &mut StdRng,
    tile_size: f32,
    available_positions: &mut Vec<Vec3>,
    door_position: Vec3,
    spawn_position: Vec3,
    exit_position: Option<Vec3>,
) {
    retain_valid_positions(
        available_positions,
        tile_size,
        door_position,
        spawn_position,
        exit_position,
    );

    let planned_props = definition.prop_plan.total();
    debug!(
        "⚙️ 關卡{} 道具佈置：樹木={}, 岩石={}, 箱子={}, 總數={}",
        definition.index + 1,
        definition.prop_plan.trees,
        definition.prop_plan.rocks,
        definition.prop_plan.crates,
        planned_props
    );

    let trees = sample_positions(available_positions, rng, definition.prop_plan.trees);
    let rocks = sample_positions(available_positions, rng, definition.prop_plan.rocks);
    let crates = sample_positions(available_positions, rng, definition.prop_plan.crates);

    for (index, position) in trees.into_iter().enumerate() {
        commands.spawn((
            LevelEntity,
            Sprite::from_image(assets.tree.clone()),
            Transform::from_translation(Vec3::new(position.x, position.y, ENVIRONMENT_PROP_Z))
                .with_scale(Vec3::splat(ENVIRONMENT_PROP_SCALE)),
            EnvironmentProp::blocking(),
            Name::new(format!("Level{}Tree{}", definition.index + 1, index + 1)),
        ));
    }

    for (index, position) in rocks.into_iter().enumerate() {
        commands.spawn((
            LevelEntity,
            Sprite::from_image(assets.rock.clone()),
            Transform::from_translation(Vec3::new(position.x, position.y, ENVIRONMENT_PROP_Z))
                .with_scale(Vec3::splat(ENVIRONMENT_PROP_SCALE)),
            EnvironmentProp::blocking(),
            Name::new(format!("Level{}Rock{}", definition.index + 1, index + 1)),
        ));
    }

    for (index, position) in crates.into_iter().enumerate() {
        commands.spawn((
            LevelEntity,
            Sprite::from_image(assets.crate_prop.clone()),
            Transform::from_translation(Vec3::new(position.x, position.y, ENVIRONMENT_PROP_Z))
                .with_scale(Vec3::splat(ENVIRONMENT_PROP_SCALE)),
            EnvironmentProp::decorative(),
            Name::new(format!("Level{}Crate{}", definition.index + 1, index + 1)),
        ));
    }
}

fn spawn_enemies_for_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
    definition: &LevelDefinition,
    rng: &mut StdRng,
    tile_size: f32,
    available_positions: &mut Vec<Vec3>,
    door_position: Vec3,
    spawn_position: Vec3,
    exit_position: Option<Vec3>,
) {
    let mut needed = definition.enemy_total();

    retain_valid_positions(
        available_positions,
        tile_size,
        door_position,
        spawn_position,
        exit_position,
    );

    if available_positions.len() < needed {
        warn!(
            "可用地板數量不足以產生所有敵人 (需要{}個, 僅有{})",
            needed,
            available_positions.len()
        );
        needed = available_positions.len();
    }

    let slime_texture = asset_server.load("characters/enemies/slime.png");
    let cyclops_texture = asset_server.load("characters/enemies/cyclops.png");

    let mut spawn_index = 0usize;
    let spawn_positions = sample_positions(available_positions, rng, needed);

    let mut assigned = HashSet::new();

    for _ in 0..definition.enemy_counts.slimes {
        if spawn_index >= spawn_positions.len() {
            break;
        }
        let position = spawn_positions[spawn_index];
        spawn_index += 1;

        let tile_key = (
            (position.x / tile_size).round() as i32,
            (position.y / tile_size).round() as i32,
        );

        if !assigned.insert(tile_key) {
            continue;
        }

        spawn_slime_entity(
            commands,
            &slime_texture,
            position,
            tile_size,
            definition.index,
            spawn_index,
        );
    }

    for _ in 0..definition.enemy_counts.cyclops {
        if spawn_index >= spawn_positions.len() {
            break;
        }
        let position = spawn_positions[spawn_index];
        spawn_index += 1;

        let tile_key = (
            (position.x / tile_size).round() as i32,
            (position.y / tile_size).round() as i32,
        );

        if !assigned.insert(tile_key) {
            continue;
        }

        spawn_cyclops_entity(
            commands,
            &cyclops_texture,
            position,
            tile_size,
            definition.index,
            spawn_index,
        );
    }
}

fn spawn_slime_entity(
    commands: &mut Commands,
    texture: &Handle<Image>,
    position: Vec3,
    tile_size: f32,
    level_index: usize,
    serial: usize,
) {
    let patrol_origin = Vec3::new(position.x, position.y, 9.0);
    let patrol_range = (SLIME_PATROL_RANGE)
        .min(tile_size * 6.0)
        .max(tile_size * 1.5);
    let direction = if serial % 2 == 0 { 1.0 } else { -1.0 };

    commands.spawn((
        LevelEntity,
        Enemy,
        Slime,
        Sprite::from_image(texture.clone()),
        Transform::from_translation(patrol_origin).with_scale(Vec3::splat(SLIME_SCALE)),
        Health::new(SLIME_HEALTH),
        Attack::new(SLIME_BASE_ATTACK),
        Defense::new(SLIME_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: patrol_origin,
            range: patrol_range,
            direction,
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
        Name::new(format!("Level{}Slime{}", level_index + 1, serial)),
    ));
}

fn spawn_cyclops_entity(
    commands: &mut Commands,
    texture: &Handle<Image>,
    position: Vec3,
    tile_size: f32,
    level_index: usize,
    serial: usize,
) {
    let patrol_origin = Vec3::new(position.x, position.y, 9.0);
    let patrol_range = (CYCLOPS_PATROL_RANGE)
        .min(tile_size * 6.0)
        .max(tile_size * 2.0);
    let direction = if serial % 2 == 0 { -1.0 } else { 1.0 };

    commands.spawn((
        LevelEntity,
        Enemy,
        Cyclops,
        Sprite::from_image(texture.clone()),
        Transform::from_translation(patrol_origin).with_scale(Vec3::splat(CYCLOPS_SCALE)),
        Health::new(CYCLOPS_HEALTH),
        Attack::new(CYCLOPS_BASE_ATTACK),
        Defense::new(CYCLOPS_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: patrol_origin,
            range: patrol_range,
            direction,
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
        Name::new(format!("Level{}Cyclops{}", level_index + 1, serial)),
    ));
}

fn sample_positions(positions: &mut Vec<Vec3>, rng: &mut StdRng, count: usize) -> Vec<Vec3> {
    let mut result = Vec::new();
    for _ in 0..count {
        if positions.is_empty() {
            break;
        }
        let index = rng.gen_range(0..positions.len());
        result.push(positions.swap_remove(index));
    }
    result
}

fn retain_valid_positions(
    positions: &mut Vec<Vec3>,
    tile_size: f32,
    door_position: Vec3,
    spawn_position: Vec3,
    exit_position: Option<Vec3>,
) {
    let min_distance = tile_size * 1.5;
    positions.retain(|pos| {
        let distance_to_door = pos.truncate().distance(door_position.truncate());
        let distance_to_spawn = pos.truncate().distance(spawn_position.truncate());
        let distance_to_exit = exit_position
            .map(|exit| pos.truncate().distance(exit.truncate()))
            .unwrap_or(f32::INFINITY);

        distance_to_door > min_distance
            && distance_to_spawn > min_distance
            && distance_to_exit > min_distance
    });
}
