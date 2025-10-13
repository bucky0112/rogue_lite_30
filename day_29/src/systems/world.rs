use crate::components::{AttackReticle, Enemy, Player, PlayerDead, level::LevelEntity, world::*};
use crate::constants::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::prelude::*;

pub fn spawn_world_floor_and_bounds(mut commands: Commands, room_assets: Res<RoomAssets>) {
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let min_x_tile = -WORLD_HALF_WIDTH_TILES - WORLD_BOUNDARY_PADDING_TILES;
    let max_x_tile = WORLD_HALF_WIDTH_TILES + WORLD_BOUNDARY_PADDING_TILES;
    let min_y_tile = -WORLD_HALF_HEIGHT_TILES - WORLD_BOUNDARY_PADDING_TILES;
    let max_y_tile = WORLD_HALF_HEIGHT_TILES + WORLD_BOUNDARY_PADDING_TILES;

    for y in min_y_tile..=max_y_tile {
        for x in min_x_tile..=max_x_tile {
            commands.spawn((
                Sprite::from_image(room_assets.floor_outdoor.clone()),
                Transform::from_translation(Vec3::new(
                    x as f32 * tile_size,
                    y as f32 * tile_size,
                    WORLD_OUTDOOR_FLOOR_Z,
                ))
                .with_scale(Vec3::splat(PLAYER_SCALE)),
            ));
        }
    }

    let min = Vec2::new(min_x_tile as f32 * tile_size, min_y_tile as f32 * tile_size);
    let max = Vec2::new(max_x_tile as f32 * tile_size, max_y_tile as f32 * tile_size);
    commands.insert_resource(WorldBounds::new(min, max));
}

pub fn spawn_room(mut commands: Commands, room_assets: Res<RoomAssets>) {
    let mut rng = rand::thread_rng();
    let room_type_choice = rng.gen_range(0..4);

    match room_type_choice {
        0 => {
            // 基本矩形房間 (25% 機率)
            let room_width = rng.gen_range(8..15);
            let room_height = rng.gen_range(6..10);
            // 置中策略：以 (0,0) 為視覺中心
            let room_x = -(room_width as i32) / 2;
            let room_y = -(room_height as i32) / 2;
            generate_room_tiles(
                &mut commands,
                &room_assets,
                room_width,
                room_height,
                room_x,
                room_y,
                true,
            );
            println!("Basic rectangular room generated");
        }
        1 => {
            // L 形房間 (25% 機率)
            let compound_room = generate_l_shape_room(&mut rng);
            spawn_compound_room(&mut commands, &room_assets, compound_room);
            println!("L-shaped room generated");
        }
        2 => {
            // T 形房間 (25% 機率)
            let compound_room = generate_t_shape_room(&mut rng);
            spawn_compound_room(&mut commands, &room_assets, compound_room);
            println!("T-shaped room generated");
        }
        3 => {
            // 十字形房間 (25% 機率)
            let compound_room = generate_cross_shape_room(&mut rng);
            spawn_compound_room(&mut commands, &room_assets, compound_room);
            println!("Cross-shaped room generated");
        }
        _ => {}
    }
}

pub fn spawn_environment_props(mut commands: Commands, environment_assets: Res<EnvironmentAssets>) {
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let scale = Vec3::splat(ENVIRONMENT_PROP_SCALE);

    let placements = [
        (
            environment_assets.tree.clone(),
            IVec2::new(-WORLD_HALF_WIDTH_TILES + 2, WORLD_HALF_HEIGHT_TILES - 3),
            true,
            "TreeNorthwest",
        ),
        (
            environment_assets.tree.clone(),
            IVec2::new(WORLD_HALF_WIDTH_TILES - 3, WORLD_HALF_HEIGHT_TILES - 5),
            true,
            "TreeNortheast",
        ),
        (
            environment_assets.rock.clone(),
            IVec2::new(-WORLD_HALF_WIDTH_TILES + 4, -WORLD_HALF_HEIGHT_TILES + 4),
            true,
            "RockSouthwest",
        ),
        (
            environment_assets.crate_prop.clone(),
            IVec2::new(WORLD_HALF_WIDTH_TILES - 5, -WORLD_HALF_HEIGHT_TILES + 6),
            false,
            "CrateSoutheast",
        ),
        (
            environment_assets.rock.clone(),
            IVec2::new(0, WORLD_HALF_HEIGHT_TILES - 2),
            true,
            "RockNorth",
        ),
    ];

    for (handle, tile_pos, blocks_movement, label) in placements {
        let world_position = Vec3::new(
            tile_pos.x as f32 * tile_size,
            tile_pos.y as f32 * tile_size,
            ENVIRONMENT_PROP_Z,
        );

        let prop = if blocks_movement {
            EnvironmentProp::blocking()
        } else {
            EnvironmentProp::decorative()
        };

        commands.spawn((
            Sprite::from_image(handle),
            Transform::from_translation(world_position).with_scale(scale),
            prop,
            LevelEntity,
            Name::new(label),
        ));
    }
}

pub fn enforce_world_bounds_system(
    world_bounds: Option<Res<WorldBounds>>,
    mut queries: ParamSet<(
        Query<
            &mut Transform,
            (
                With<Player>,
                Without<PlayerDead>,
                Without<Enemy>,
                Without<AttackReticle>,
            ),
        >,
        Query<&mut Transform, (With<Enemy>, Without<Player>, Without<AttackReticle>)>,
        Query<&mut Transform, (With<AttackReticle>, Without<Player>, Without<Enemy>)>,
    )>,
) {
    let Some(bounds) = world_bounds else {
        return;
    };

    if let Ok(mut transform) = queries.p0().single_mut() {
        let clamped = bounds.clamp_translation(transform.translation);
        transform.translation = clamped;
    }

    for mut transform in queries.p1().iter_mut() {
        let clamped = bounds.clamp_translation(transform.translation);
        transform.translation = clamped;
    }

    for mut transform in queries.p2().iter_mut() {
        let clamped = bounds.clamp_translation(transform.translation);
        transform.translation = clamped;
    }
}

pub fn generate_room_tiles(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    width: usize,
    height: usize,
    start_x: i32,
    start_y: i32,
    should_generate_door: bool,
) {
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let total_height = height + 1; // 額外一列用於南牆外側
    let mut door_world_position: Option<Vec3> = None;

    for y in 0..total_height {
        for x in 0..width {
            let world_x = (start_x + x as i32) as f32 * tile_size;
            let world_y = (start_y + y as i32 - 1) as f32 * tile_size; // 將南牆外側往下對齊

            let (tile_type, image_handle) = if y == total_height - 1 {
                // 北牆（面向玩家）
                if x == 0 {
                    (
                        RoomTileType::WallNInnerCornerW,
                        room_assets.wall_n_inner_corner_w.clone(),
                    )
                } else if x == width - 1 {
                    (
                        RoomTileType::WallNInnerCornerE,
                        room_assets.wall_n_inner_corner_e.clone(),
                    )
                } else {
                    (
                        RoomTileType::WallNInnerMid,
                        room_assets.wall_n_inner_mid.clone(),
                    )
                }
            } else if y == 1 {
                // 南牆內側
                if x == 0 {
                    (
                        RoomTileType::WallSInnerCapL,
                        room_assets.wall_s_inner_cap_l.clone(),
                    )
                } else if x == width - 1 {
                    (
                        RoomTileType::WallSInnerCapR,
                        room_assets.wall_s_inner_cap_r.clone(),
                    )
                } else {
                    (
                        RoomTileType::WallSInnerMid,
                        room_assets.wall_s_inner_mid.clone(),
                    )
                }
            } else if y == 0 {
                // 南牆外側
                let door_x = width / 2;
                if x == 0 {
                    (
                        RoomTileType::WallSOuterCapL,
                        room_assets.wall_s_outer_cap_l.clone(),
                    )
                } else if x == width - 1 {
                    (
                        RoomTileType::WallSOuterCapR,
                        room_assets.wall_s_outer_cap_r.clone(),
                    )
                } else if x == door_x && should_generate_door {
                    (RoomTileType::DoorClosed, room_assets.door_closed.clone())
                } else {
                    (
                        RoomTileType::WallSOuterMid,
                        room_assets.wall_s_outer_mid.clone(),
                    )
                }
            } else if x == 0 {
                (RoomTileType::WallWSide, room_assets.wall_w_side.clone())
            } else if x == width - 1 {
                (RoomTileType::WallESide, room_assets.wall_e_side.clone())
            } else {
                (RoomTileType::Floor, room_assets.floor_indoor.clone())
            };

            let mut entity_commands = commands.spawn((
                Sprite::from_image(image_handle),
                Transform::from_translation(Vec3::new(world_x, world_y, Z_LAYER_GRID))
                    .with_scale(Vec3::splat(PLAYER_SCALE)),
                RoomTile { tile_type },
                LevelEntity,
            ));

            if matches!(tile_type, RoomTileType::DoorClosed | RoomTileType::DoorOpen) {
                entity_commands.insert(Door {
                    is_open: tile_type == RoomTileType::DoorOpen,
                });
                door_world_position = Some(Vec3::new(world_x, world_y, Z_LAYER_GRID));
            }
        }
    }

    if should_generate_door {
        if let Some(door_pos) = door_world_position {
            let outdoor_depth_tiles = 5;
            let outdoor_extra_width = 4; // 讓外部區域略寬於門口
            let spawn_offset_tiles = 2; // 玩家出生點距離門的距離

            let half_width = (width as i32 + outdoor_extra_width) / 2;

            for depth in 1..=outdoor_depth_tiles {
                let exterior_y = door_pos.y - tile_size * depth as f32;

                for offset in -half_width..=half_width {
                    let exterior_x = door_pos.x + offset as f32 * tile_size;

                    commands.spawn((
                        Sprite::from_image(room_assets.floor_outdoor.clone()),
                        Transform::from_translation(Vec3::new(
                            exterior_x,
                            exterior_y,
                            Z_LAYER_GRID,
                        ))
                        .with_scale(Vec3::splat(PLAYER_SCALE)),
                        RoomTile {
                            tile_type: RoomTileType::FloorOutdoor,
                        },
                        LevelEntity,
                    ));
                }
            }

            let spawn_y = door_pos.y - tile_size * spawn_offset_tiles as f32;
            commands.insert_resource(EntranceLocation::new(Vec3::new(door_pos.x, spawn_y, 10.0)));
        }
    }
}

fn generate_l_shape_room(rng: &mut impl Rng) -> CompoundRoom {
    let main_width = rng.gen_range(6..10);
    let main_height = rng.gen_range(8..12);
    let extension_width = rng.gen_range(5..9);
    let extension_height = rng.gen_range(4..7).min(main_height - 1);

    // 主房間（垂直部分）
    let main_rect = RoomRect {
        x: -(main_width as i32) / 2,
        y: -(main_height as i32) / 2,
        width: main_width,
        height: main_height,
    };

    // 擴展房間（水平部分）- 與主房間重疊 1 格確保連通
    let extension_rect = RoomRect {
        x: main_rect.x + (main_width as i32) - 1,
        y: main_rect.y,
        width: extension_width + 1,
        height: extension_height,
    };

    CompoundRoom {
        rectangles: vec![main_rect, extension_rect],
        room_type: CompoundRoomType::LShape,
    }
}

fn generate_t_shape_room(rng: &mut impl Rng) -> CompoundRoom {
    let beam_width = rng.gen_range(8..12);
    let beam_height = rng.gen_range(4..6);
    let pillar_width = rng.gen_range(4..6);
    let pillar_height = rng.gen_range(6..9);

    // 上橫梁
    let top_beam = RoomRect {
        x: -(beam_width as i32) / 2,
        y: -(pillar_height as i32) / 2 + (pillar_height as i32) - 1,
        width: beam_width,
        height: beam_height,
    };

    // 下豎梁
    let bottom_pillar = RoomRect {
        x: -(pillar_width as i32) / 2,
        y: -(pillar_height as i32) / 2,
        width: pillar_width,
        height: pillar_height,
    };

    CompoundRoom {
        rectangles: vec![top_beam, bottom_pillar],
        room_type: CompoundRoomType::TShape,
    }
}

fn generate_cross_shape_room(rng: &mut impl Rng) -> CompoundRoom {
    let h_beam_width = rng.gen_range(8..12);
    let h_beam_height = rng.gen_range(4..6);
    let v_beam_width = rng.gen_range(4..6);
    let v_beam_height = rng.gen_range(8..12);

    // 水平橫梁
    let horizontal_beam = RoomRect {
        x: -(h_beam_width as i32) / 2,
        y: -(h_beam_height as i32) / 2,
        width: h_beam_width,
        height: h_beam_height,
    };

    // 垂直豎梁
    let vertical_beam = RoomRect {
        x: -(v_beam_width as i32) / 2,
        y: -(v_beam_height as i32) / 2,
        width: v_beam_width,
        height: v_beam_height,
    };

    CompoundRoom {
        rectangles: vec![horizontal_beam, vertical_beam],
        room_type: CompoundRoomType::Cross,
    }
}

pub fn spawn_compound_room(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    compound_room: CompoundRoom,
) {
    // 找到y座標最小的矩形（最下面的房間）
    let bottommost_rect_index = compound_room
        .rectangles
        .iter()
        .enumerate()
        .min_by_key(|(_, rect)| rect.y)
        .map(|(index, _)| index)
        .unwrap_or(0);

    // 只在最下面的矩形生成門
    for (index, rect) in compound_room.rectangles.iter().enumerate() {
        let should_generate_door = index == bottommost_rect_index;
        generate_room_tiles(
            commands,
            room_assets,
            rect.width,
            rect.height,
            rect.x,
            rect.y,
            should_generate_door,
        );
    }

    // 生成連接走廊
    generate_corridors(commands, room_assets, &compound_room);

    // 生成複合房間實體
    commands.spawn((LevelEntity, compound_room));
}

fn generate_corridors(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    compound_room: &CompoundRoom,
) {
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;

    match compound_room.room_type {
        CompoundRoomType::LShape => {
            carve_l_shape_corridor(commands, room_assets, compound_room, tile_size);
        }
        CompoundRoomType::TShape => {
            let top_beam = &compound_room.rectangles[0];
            let bottom_pillar = &compound_room.rectangles[1];
            let overlap_x_start = top_beam.x.max(bottom_pillar.x);
            let overlap_x_end = (top_beam.x + top_beam.width as i32)
                .min(bottom_pillar.x + bottom_pillar.width as i32);
            let overlap_y = top_beam.y;

            for x in overlap_x_start..overlap_x_end {
                spawn_corridor_floor_tile(commands, room_assets, tile_size, x, overlap_y);
            }
        }
        CompoundRoomType::Cross => {
            let h_beam = &compound_room.rectangles[0];
            let v_beam = &compound_room.rectangles[1];
            let overlap_x_start = h_beam.x.max(v_beam.x);
            let overlap_x_end =
                (h_beam.x + h_beam.width as i32).min(v_beam.x + v_beam.width as i32);
            let overlap_y_start = h_beam.y.max(v_beam.y);
            let overlap_y_end =
                (h_beam.y + h_beam.height as i32).min(v_beam.y + v_beam.height as i32);

            for x in overlap_x_start..overlap_x_end {
                for y in overlap_y_start..overlap_y_end {
                    spawn_corridor_floor_tile(commands, room_assets, tile_size, x, y);
                }
            }
        }
    }
}

fn carve_l_shape_corridor(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    compound_room: &CompoundRoom,
    tile_size: f32,
) {
    if compound_room.rectangles.len() < 2 {
        return;
    }

    let main_rect = &compound_room.rectangles[0];
    let ext_rect = &compound_room.rectangles[1];

    let corridor_x_left = main_rect.x + main_rect.width as i32 - 1;
    let corridor_x_right = ext_rect.x;

    let overlap_y_start = ext_rect.y.max(main_rect.y);
    let overlap_y_end =
        (ext_rect.y + ext_rect.height as i32 - 1).min(main_rect.y + main_rect.height as i32 - 1);

    if overlap_y_start > overlap_y_end {
        return;
    }

    let corridor_center_y = (overlap_y_start + overlap_y_end) / 2;
    let y_min = (corridor_center_y - 1).max(overlap_y_start);
    let y_max = (corridor_center_y + 1).min(overlap_y_end);

    let rect_min_x = main_rect.x.min(ext_rect.x);
    let rect_max_x =
        (main_rect.x + main_rect.width as i32 - 1).max(ext_rect.x + ext_rect.width as i32 - 1);

    let mut x_min = corridor_x_left.min(corridor_x_right) - 1;
    let mut x_max = corridor_x_left.max(corridor_x_right) + 1;

    x_min = x_min.max(rect_min_x);
    x_max = x_max.min(rect_max_x);

    if x_min > x_max || y_min > y_max {
        return;
    }

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            spawn_corridor_floor_tile(commands, room_assets, tile_size, x, y);
        }
    }
}

fn spawn_corridor_floor_tile(
    commands: &mut Commands,
    room_assets: &RoomAssets,
    tile_size: f32,
    x: i32,
    y: i32,
) {
    commands.spawn((
        Sprite::from_image(room_assets.floor_indoor.clone()),
        Transform::from_translation(Vec3::new(
            x as f32 * tile_size,
            y as f32 * tile_size,
            Z_LAYER_GRID + 0.2,
        ))
        .with_scale(Vec3::splat(PLAYER_SCALE)),
        RoomTile {
            tile_type: RoomTileType::Floor,
        },
        CorridorTile,
        LevelEntity,
    ));
}
