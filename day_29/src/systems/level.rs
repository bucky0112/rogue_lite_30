use std::collections::HashSet;

use crate::components::{
    Attack, BossWizard, BossWizardStaff, Chest, ChestContents, Cyclops, CyclopsCharge, Defense,
    Enemy, EnemyAIState, EnemyAlert, EnemyAttack, EnemyBehaviorState, EnemyPatrol, EnemySpeeds,
    Health, PickupEffect, ShieldKind, Slime, Spider, WeaponKind,
    level::{LevelEntity, LevelExitDoor},
    player::{InputVector, Player, PlayerDead, Velocity},
    world::{CorridorTile, Door, EnvironmentProp, RoomTile, RoomTileType},
};
use crate::constants::*;
use crate::resources::{
    EntranceLocation, EnvironmentAssets, LevelBuildContext, LevelDefinition, LevelExitAssets,
    LevelState, PendingLevelRewards, RoomAssets,
};
use crate::systems::EnemyDefeatedEvent;
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{Node, PositionType, Val};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom, thread_rng};

#[derive(Event, Debug, Clone, Copy)]
pub struct LevelAdvanceRequestEvent {
    pub target_level: usize,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct LevelLoadedEvent {
    pub index: usize,
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct PlayerCombatSnapshot {
    attack: i32,
    defense: i32,
    max_health: i32,
}

impl PlayerCombatSnapshot {
    fn from_components(attack: &Attack, defense: &Defense, health: &Health) -> Self {
        Self {
            attack: attack.value().max(1),
            defense: defense.value().max(0),
            max_health: health.max.max(1),
        }
    }
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
            info!("📦 Already at the final level; cannot advance further");
            continue;
        };

        if event.target_level != expected_next {
            warn!(
                "Ignoring invalid level request: target={}, expected={}",
                event.target_level, expected_next
            );
            continue;
        }

        info!("🌀 Preparing to load level {}", event.target_level + 1);
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
    asset_server: Res<AssetServer>,
    door_query: Query<&Transform, (With<Door>, With<LevelEntity>)>,
    tile_query: Query<(&Transform, &RoomTile, Option<&CorridorTile>), With<LevelEntity>>,
    mut player_query: Query<
        (
            &mut Transform,
            Option<&mut Velocity>,
            Option<&mut InputVector>,
        ),
        (With<Player>, Without<PlayerDead>, Without<LevelEntity>),
    >,
    player_stats_query: Query<
        (&Attack, &Defense, &Health),
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
        warn!("No entrance door found; skipping level initialization");
        return;
    };

    let door_position = door_transform.translation;

    let player_snapshot = player_stats_query
        .get_single()
        .ok()
        .map(|(attack, defense, health)| {
            PlayerCombatSnapshot::from_components(attack, defense, health)
        });

    let tile_samples: Vec<(Vec3, RoomTileType, bool)> = tile_query
        .iter()
        .map(|(transform, tile, corridor)| {
            (transform.translation, tile.tile_type, corridor.is_some())
        })
        .collect();

    let corridor_tiles: HashSet<(i32, i32)> = tile_samples
        .iter()
        .filter(|(_, _, is_corridor)| *is_corridor)
        .map(|(position, _, _)| {
            (
                (position.x / tile_size).round() as i32,
                (position.y / tile_size).round() as i32,
            )
        })
        .collect();

    let spawn_position = compute_player_spawn_position(&tile_samples, tile_size, door_position);
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

    let mut floor_positions: Vec<Vec3> = tile_samples
        .iter()
        .filter_map(|(position, tile, is_corridor)| match tile {
            RoomTileType::Floor | RoomTileType::FloorOutdoor if !is_corridor => Some(*position),
            _ => None,
        })
        .collect();

    let floor_tiles: HashSet<(i32, i32)> = tile_samples
        .iter()
        .filter_map(|(position, tile, _)| match tile {
            RoomTileType::Floor | RoomTileType::FloorOutdoor => Some((
                (position.x / tile_size).round() as i32,
                (position.y / tile_size).round() as i32,
            )),
            _ => None,
        })
        .collect();

    floor_positions.retain(|pos| pos.y > door_position.y + tile_size * 0.25);

    floor_positions.shuffle(&mut rng);

    let computed_anchor = compute_portal_anchor(&tile_samples, tile_size).or(Some(Vec3::new(
        door_position.x + tile_size * 2.0,
        spawn_position.y,
        spawn_position.z,
    )));

    let next_level = level_state.next_index();
    let portal_anchor = computed_anchor;
    let exit_position =
        portal_anchor.map(|anchor| Vec3::new(anchor.x, anchor.y + tile_size * 0.5, 11.0));

    let candidate_positions = floor_positions.clone();

    spawn_level_chests(
        &mut commands,
        asset_server.as_ref(),
        &mut floor_positions,
        &floor_tiles,
        &candidate_positions,
        door_position,
        spawn_position,
        exit_position,
        tile_size,
    );

    spawn_environment_props_for_level(
        &mut commands,
        &floor_tiles,
        &corridor_tiles,
        &environment_assets,
        definition,
        &mut rng,
        tile_size,
        &mut floor_positions,
        door_position,
        spawn_position,
        exit_position,
    );

    let is_final_level = definition.index + 1 >= level_state.definition_count().max(1);

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
        player_snapshot,
        is_final_level,
    );

    info!(
        "🌌 Level {} - {} ready (enemies={}, props={})",
        definition.index + 1,
        definition.name,
        definition.enemy_total(),
        definition.prop_plan.total()
    );

    commands.insert_resource(PendingLevelRewards {
        level_index: definition.index,
        portal_anchor,
        tile_size,
        target_level: next_level,
        rewards_spawned: false,
        rewards_available: definition.enemy_counts.boss_wizards > 0,
    });

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
        "🚪 Spawned exit to level {} at ({:.1}, {:.1})",
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

fn compute_player_spawn_position(
    tiles: &[(Vec3, RoomTileType, bool)],
    tile_size: f32,
    door_position: Vec3,
) -> Vec3 {
    let door_tile_x = (door_position.x / tile_size).round() as i32;
    let door_tile_y = (door_position.y / tile_size).round() as i32;

    let mut best: Option<(Vec3, f32)> = None;

    for (position, tile, is_corridor) in tiles {
        if *is_corridor {
            continue;
        }

        if !matches!(tile, RoomTileType::Floor | RoomTileType::FloorOutdoor) {
            continue;
        }

        // Prefer floor tiles on or inside the room relative to the door.
        let tile_x = (position.x / tile_size).round() as i32;
        let tile_y = (position.y / tile_size).round() as i32;

        if tile_y <= door_tile_y {
            continue;
        }

        let alignment = (tile_x - door_tile_x).abs() as f32;
        let distance = door_position.truncate().distance(position.truncate());

        let score = alignment * tile_size * 0.25 + distance;

        if best.map_or(true, |(_, best_score)| score < best_score) {
            best = Some((*position, score));
        }
    }

    if let Some((position, _)) = best {
        Vec3::new(position.x, position.y, 10.0)
    } else {
        Vec3::new(door_position.x, door_position.y + tile_size, 10.0)
    }
}

fn compute_portal_anchor(tiles: &[(Vec3, RoomTileType, bool)], tile_size: f32) -> Option<Vec3> {
    let mut top_y = f32::NEG_INFINITY;
    let mut x_sum = 0.0;
    let mut count = 0;
    let tolerance = tile_size * 0.1;

    for (position, tile, is_corridor) in tiles {
        if *tile != RoomTileType::Floor || *is_corridor {
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
    floor_tiles: &HashSet<(i32, i32)>,
    corridor_tiles: &HashSet<(i32, i32)>,
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
        "⚙️ Level {} props: trees={}, rocks={}, crates={}, total={}",
        definition.index + 1,
        definition.prop_plan.trees,
        definition.prop_plan.rocks,
        definition.prop_plan.crates,
        planned_props
    );

    let trees: Vec<Vec3> = sample_positions(available_positions, rng, definition.prop_plan.trees)
        .into_iter()
        .filter(|pos| {
            !is_corridor_position(*pos, tile_size, corridor_tiles)
                && prop_position_valid(
                    *pos,
                    tile_size,
                    floor_tiles,
                    door_position,
                    spawn_position,
                    exit_position,
                )
        })
        .collect();
    let rocks: Vec<Vec3> = sample_positions(available_positions, rng, definition.prop_plan.rocks)
        .into_iter()
        .filter(|pos| {
            !is_corridor_position(*pos, tile_size, corridor_tiles)
                && prop_position_valid(
                    *pos,
                    tile_size,
                    floor_tiles,
                    door_position,
                    spawn_position,
                    exit_position,
                )
        })
        .collect();
    let crates: Vec<Vec3> = sample_positions(available_positions, rng, definition.prop_plan.crates)
        .into_iter()
        .filter(|pos| {
            !is_corridor_position(*pos, tile_size, corridor_tiles)
                && prop_position_valid(
                    *pos,
                    tile_size,
                    floor_tiles,
                    door_position,
                    spawn_position,
                    exit_position,
                )
        })
        .collect();
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
    player_stats: Option<PlayerCombatSnapshot>,
    final_level: bool,
) {
    let mut needed = definition.enemy_counts.slimes
        + definition.enemy_counts.cyclops
        + definition.enemy_counts.spiders;

    retain_valid_positions(
        available_positions,
        tile_size,
        door_position,
        spawn_position,
        exit_position,
    );

    if available_positions.len() < needed {
        warn!(
            "Not enough floor tiles for all enemies (needs {}, only {})",
            needed,
            available_positions.len()
        );
        needed = available_positions.len();
    }

    let slime_texture = asset_server.load("characters/enemies/slime.png");
    let spider_texture = asset_server.load("characters/enemies/spider.png");
    let cyclops_texture = asset_server.load("characters/enemies/cyclops.png");

    let floor_tiles: HashSet<(i32, i32)> = available_positions
        .iter()
        .map(|position| {
            (
                (position.x / tile_size).round() as i32,
                (position.y / tile_size).round() as i32,
            )
        })
        .collect();

    let mut spawn_positions = sample_positions(available_positions, rng, needed);

    let mut assigned = HashSet::new();

    fn take_position<F>(
        positions: &mut Vec<Vec3>,
        assigned: &mut HashSet<(i32, i32)>,
        tile_size: f32,
        predicate: F,
    ) -> Option<Vec3>
    where
        F: Fn(Vec3) -> bool,
    {
        let mut index = positions.len();
        while index > 0 {
            index -= 1;
            let position = positions[index];
            if !predicate(position) {
                continue;
            }

            let tile_key = (
                (position.x / tile_size).round() as i32,
                (position.y / tile_size).round() as i32,
            );

            if assigned.contains(&tile_key) {
                continue;
            }

            positions.swap_remove(index);
            assigned.insert(tile_key);
            return Some(position);
        }

        None
    }

    for serial in 0..definition.enemy_counts.slimes {
        let Some(position) =
            take_position(&mut spawn_positions, &mut assigned, tile_size, |_| true)
        else {
            break;
        };

        spawn_slime_entity(
            commands,
            &slime_texture,
            position,
            tile_size,
            definition.index,
            serial + 1,
        );
    }

    let entrance_position = spawn_position.truncate();
    let door_position_2d = door_position.truncate();
    let spider_entrance_buffer = tile_size * 4.0;

    for serial in 0..definition.enemy_counts.spiders {
        let position = take_position(
            &mut spawn_positions,
            &mut assigned,
            tile_size,
            |candidate| {
                let candidate_2d = candidate.truncate();
                candidate_2d.distance(entrance_position) >= spider_entrance_buffer
                    && candidate_2d.distance(door_position_2d) >= spider_entrance_buffer
                    && has_horizontal_clearance(candidate, tile_size, &floor_tiles)
            },
        )
        .or_else(|| {
            take_position(
                &mut spawn_positions,
                &mut assigned,
                tile_size,
                |candidate| {
                    let candidate_2d = candidate.truncate();
                    candidate_2d.distance(entrance_position) >= spider_entrance_buffer
                        && candidate_2d.distance(door_position_2d) >= spider_entrance_buffer
                },
            )
        });

        let Some(position) = position else {
            break;
        };

        spawn_spider_entity(
            commands,
            &spider_texture,
            position,
            tile_size,
            definition.index,
            serial + 1,
            &floor_tiles,
        );
    }

    for serial in 0..definition.enemy_counts.cyclops {
        let Some(position) =
            take_position(&mut spawn_positions, &mut assigned, tile_size, |_| true)
        else {
            break;
        };

        spawn_cyclops_entity(
            commands,
            &cyclops_texture,
            position,
            tile_size,
            definition.index,
            serial + 1,
        );
    }

    if definition.enemy_counts.boss_wizards > 0 {
        let Some(exit_location) = exit_position else {
            warn!(
                "Level {} defines a wizard boss but is missing portal coordinates; skipping boss",
                definition.index + 1
            );
            return;
        };

        let body_texture = asset_server.load(WIZARD_BOSS_SPRITE_PATH);
        let staff_texture = asset_server.load(WIZARD_BOSS_STAFF_SPRITE_PATH);

        for serial in 0..definition.enemy_counts.boss_wizards {
            let offset_multiplier = serial as f32;
            let spawn_position = Vec3::new(
                exit_location.x,
                exit_location.y - tile_size * (1.2 + offset_multiplier * 0.6),
                9.0,
            );

            spawn_boss_wizard_entity(
                commands,
                &body_texture,
                &staff_texture,
                spawn_position,
                definition.index,
                serial + 1,
                player_stats,
                final_level,
            );
        }
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
        EnemyAttack {
            radius: CYCLOPS_ATTACK_RADIUS,
            cooldown: {
                let mut timer = Timer::from_seconds(CYCLOPS_ATTACK_COOLDOWN, TimerMode::Repeating);
                timer.set_elapsed(timer.duration());
                timer
            },
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

fn spawn_spider_entity(
    commands: &mut Commands,
    texture: &Handle<Image>,
    position: Vec3,
    tile_size: f32,
    level_index: usize,
    serial: usize,
    floor_tiles: &HashSet<(i32, i32)>,
) {
    let (patrol_origin, patrol_range) =
        resolve_spider_patrol_bounds(position, tile_size, floor_tiles);
    let direction = if serial % 2 == 0 { 1.0 } else { -1.0 };

    commands.spawn((
        LevelEntity,
        Enemy,
        Spider,
        Sprite::from_image(texture.clone()),
        Transform::from_translation(patrol_origin).with_scale(Vec3::splat(SPIDER_SCALE)),
        Health::new(SPIDER_HEALTH),
        Attack::new(SPIDER_BASE_ATTACK),
        Defense::new(SPIDER_BASE_DEFENSE),
        EnemyAIState {
            state: EnemyBehaviorState::Patrolling,
        },
        EnemyPatrol {
            origin: patrol_origin,
            range: patrol_range,
            direction,
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
        Name::new(format!("Level{}Spider{}", level_index + 1, serial)),
    ));
}

fn spawn_boss_wizard_entity(
    commands: &mut Commands,
    body_texture: &Handle<Image>,
    staff_texture: &Handle<Image>,
    position: Vec3,
    level_index: usize,
    serial: usize,
    player_stats: Option<PlayerCombatSnapshot>,
    final_boss: bool,
) {
    let patrol_origin = Vec3::new(position.x, position.y, 9.0);
    let (boss_health, boss_attack, boss_defense) =
        wizard_boss_stats(level_index, final_boss, player_stats);

    let boss_entity = commands
        .spawn((
            LevelEntity,
            Enemy,
            BossWizard,
            Sprite::from_image(body_texture.clone()),
            Transform::from_translation(patrol_origin).with_scale(Vec3::splat(WIZARD_BOSS_SCALE)),
            Health::new(boss_health),
            Attack::new(boss_attack),
            Defense::new(boss_defense),
            EnemyAIState {
                state: EnemyBehaviorState::Patrolling,
            },
            EnemyPatrol {
                origin: patrol_origin,
                range: WIZARD_BOSS_PATROL_RANGE,
                direction: 1.0,
            },
            EnemyAlert {
                trigger_radius: WIZARD_BOSS_ALERT_RADIUS,
                leash_radius: WIZARD_BOSS_LEASH_RADIUS,
            },
            EnemySpeeds {
                patrol: WIZARD_BOSS_PATROL_SPEED,
                chase: WIZARD_BOSS_CHASE_SPEED,
            },
            EnemyAttack {
                radius: WIZARD_BOSS_ATTACK_RADIUS,
                cooldown: {
                    let mut timer =
                        Timer::from_seconds(WIZARD_BOSS_ATTACK_COOLDOWN, TimerMode::Repeating);
                    timer.set_elapsed(timer.duration());
                    timer
                },
            },
            Name::new(format!("Level{}WizardBoss{}", level_index + 1, serial)),
        ))
        .id();

    commands.entity(boss_entity).with_children(|parent| {
        parent.spawn((
            BossWizardStaff,
            Sprite::from_image(staff_texture.clone()),
            Transform::from_translation(Vec3::new(
                WIZARD_BOSS_STAFF_OFFSET_X,
                WIZARD_BOSS_STAFF_OFFSET_Y,
                WIZARD_BOSS_STAFF_OFFSET_Z,
            ))
            .with_scale(Vec3::splat(WIZARD_BOSS_STAFF_SCALE)),
            Name::new(format!("Level{}WizardStaff{}", level_index + 1, serial)),
        ));
    });
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

fn has_horizontal_clearance(
    position: Vec3,
    tile_size: f32,
    floor_tiles: &HashSet<(i32, i32)>,
) -> bool {
    let tile_x = (position.x / tile_size).round() as i32;
    let tile_y = (position.y / tile_size).round() as i32;

    let left = (tile_x - 1, tile_y);
    let right = (tile_x + 1, tile_y);

    floor_tiles.contains(&left) && floor_tiles.contains(&right)
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

fn wizard_boss_stats(
    level_index: usize,
    final_boss: bool,
    player_stats: Option<PlayerCombatSnapshot>,
) -> (i32, i32, i32) {
    let multiplier = if final_boss { 1.3 } else { 1.1 };

    if let Some(stats) = player_stats {
        let attack = ((stats.attack as f32) * multiplier).ceil() as i32;
        let defense = ((stats.defense as f32) * multiplier).ceil() as i32;
        let health = ((stats.max_health as f32) * multiplier).ceil() as i32;
        return (health.max(1), attack.max(1), defense.max(0));
    }

    let idx = level_index.min(PLAYER_MAX_LEVEL);
    let player_attack = PLAYER_LEVEL_BASE_ATTACK[idx];
    let player_defense = PLAYER_LEVEL_BASE_DEFENSE[idx];
    let player_health = PLAYER_INITIAL_HEALTH;

    let attack = ((player_attack as f32) * multiplier).ceil() as i32;
    let defense = ((player_defense as f32) * multiplier).ceil() as i32;
    let health = ((player_health as f32) * multiplier).ceil() as i32;

    (health.max(1), attack.max(1), defense.max(0))
}

fn prop_position_valid(
    position: Vec3,
    tile_size: f32,
    floor_tiles: &HashSet<(i32, i32)>,
    door_position: Vec3,
    spawn_position: Vec3,
    exit_position: Option<Vec3>,
) -> bool {
    let pos2d = position.truncate();
    let buffer = tile_size * 2.5;

    if pos2d.distance(door_position.truncate()) < buffer {
        return false;
    }

    if pos2d.distance(spawn_position.truncate()) < buffer {
        return false;
    }

    if let Some(exit_pos) = exit_position {
        if pos2d.distance(exit_pos.truncate()) < buffer {
            return false;
        }
    }

    let tile_x = (position.x / tile_size).round() as i32;
    let tile_y = (position.y / tile_size).round() as i32;

    let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let accessible = neighbors
        .iter()
        .filter(|(dx, dy)| floor_tiles.contains(&(tile_x + dx, tile_y + dy)))
        .count();

    accessible >= 3
}

fn spawn_level_chests(
    commands: &mut Commands,
    asset_server: &AssetServer,
    floor_positions: &mut Vec<Vec3>,
    floor_tiles: &HashSet<(i32, i32)>,
    candidates: &[Vec3],
    door_position: Vec3,
    entrance_position: Vec3,
    exit_position: Option<Vec3>,
    tile_size: f32,
) {
    let mut rng = thread_rng();
    let mut positions = candidates.to_vec();
    positions.shuffle(&mut rng);

    let mut chosen: Vec<Vec3> = positions
        .iter()
        .copied()
        .filter(|position| {
            prop_position_valid(
                *position,
                tile_size,
                floor_tiles,
                door_position,
                entrance_position,
                exit_position,
            )
        })
        .collect();

    if chosen.len() < 3 {
        for candidate in positions.iter() {
            if chosen.contains(candidate) {
                continue;
            }
            chosen.push(*candidate);
            if chosen.len() == 3 {
                break;
            }
        }
    }

    chosen.truncate(3);

    for pos in &chosen {
        if let Some(index) = floor_positions
            .iter()
            .position(|p| p.truncate() == pos.truncate())
        {
            floor_positions.swap_remove(index);
        }
    }

    let mut slots = chosen.clone();
    slots.shuffle(&mut rng);

    let mut chest_payload: Vec<(Vec3, ChestContents, &'static str)> = Vec::new();

    if let Some(pos) = slots.get(0) {
        chest_payload.push((*pos, ChestContents::Mimic, "RewardChestMimic"));
    }

    if let Some(pos) = slots.get(1) {
        let effect = match rng.gen_range(0..3) {
            0 => PickupEffect::Heal(ITEM_HEALTH_POTION_HEAL_AMOUNT),
            1 => PickupEffect::RestoreStamina(ITEM_STAMINA_POTION_AMOUNT),
            _ => PickupEffect::CurePoison,
        };

        chest_payload.push((*pos, ChestContents::Item(effect), "RewardChestElixir"));
    }

    if let Some(pos) = slots.get(2) {
        let effect = match rng.gen_range(0..3) {
            0 => PickupEffect::Heal(ITEM_HEALTH_POTION_HEAL_AMOUNT),
            1 => PickupEffect::RestoreStamina(ITEM_STAMINA_POTION_AMOUNT),
            _ => PickupEffect::CurePoison,
        };

        chest_payload.push((*pos, ChestContents::Item(effect), "RewardChestElixir"));
    }

    for (pos, contents, label) in chest_payload {
        commands.spawn((
            LevelEntity,
            Chest::new(contents),
            Sprite::from_image(asset_server.load("items/chests/chest_closed.png")),
            Transform::from_translation(Vec3::new(pos.x, pos.y, CHEST_Z))
                .with_scale(Vec3::splat(CHEST_SCALE)),
            Name::new(label),
        ));
    }
}

pub fn spawn_rewards_on_boss_defeat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_exit_assets: Option<Res<LevelExitAssets>>,
    level_state: Res<LevelState>,
    mut rewards: ResMut<PendingLevelRewards>,
    mut events: EventReader<EnemyDefeatedEvent>,
) {
    if rewards.rewards_spawned || !rewards.rewards_available {
        return;
    }

    let mut triggered = false;
    for event in events.read() {
        if event.enemy_name == "Wizard Boss" {
            triggered = true;
        }
    }

    if !triggered {
        return;
    }

    rewards.rewards_spawned = true;

    if let Some(anchor) = rewards.portal_anchor {
        if let Some(target) = rewards.target_level {
            if let Some(assets) = level_exit_assets.as_ref() {
                spawn_level_exit_portal(
                    &mut commands,
                    assets.as_ref(),
                    anchor,
                    rewards.tile_size,
                    target,
                );
            } else {
                warn!("Missing level exit assets; cannot spawn exit portal");
            }
        }
    }

    let loot_items = match rewards.level_index {
        0 => vec![
            PickupEffect::EquipShield(ShieldKind::Level1),
            PickupEffect::EquipWeapon(WeaponKind::Level2),
        ],
        1 => vec![PickupEffect::EquipWeapon(WeaponKind::Level3)],
        2 => vec![PickupEffect::EquipWeapon(WeaponKind::Level4)],
        _ => vec![
            PickupEffect::EquipWeapon(WeaponKind::Level5),
            PickupEffect::EquipShield(ShieldKind::Level2),
        ],
    };

    if let Some(anchor) = rewards.portal_anchor {
        let spacing = rewards.tile_size * 0.7;
        let total = loot_items.len() as f32;
        let mut offset = if total > 1.0 {
            -((total - 1.0) * spacing * 0.5)
        } else {
            0.0
        };

        for effect in loot_items {
            let chest_position =
                Vec3::new(anchor.x + offset, anchor.y - rewards.tile_size * 0.8, 12.0);
            offset += spacing;

            commands.spawn((
                LevelEntity,
                Chest::new(ChestContents::Item(effect)),
                Sprite::from_image(asset_server.load("items/chests/chest_closed.png")),
                Transform::from_translation(chest_position).with_scale(Vec3::splat(CHEST_SCALE)),
                Name::new("BossRewardChest"),
            ));
        }

        let final_index = level_state.definition_count().saturating_sub(1);
        if rewards.level_index >= final_index {
            commands.spawn((
                LevelEntity,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(35.0),
                    top: Val::Percent(40.0),
                    ..Default::default()
                },
                Text::new("You Win"),
                TextFont {
                    font: asset_server.load(MENU_FONT_PATH),
                    font_size: 96.0,
                    ..Default::default()
                },
                TextColor(Color::srgba(0.98, 0.95, 0.7, 1.0)),
                Name::new("VictoryBanner"),
            ));
        }
    }

    rewards.rewards_available = false;
}

fn is_corridor_position(
    position: Vec3,
    tile_size: f32,
    corridor_tiles: &HashSet<(i32, i32)>,
) -> bool {
    let tile_key = (
        (position.x / tile_size).round() as i32,
        (position.y / tile_size).round() as i32,
    );
    corridor_tiles.contains(&tile_key)
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
