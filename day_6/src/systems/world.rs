use crate::components::world::*;
use crate::constants::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::prelude::*;

pub fn spawn_room(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                &asset_server,
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
            spawn_compound_room(&mut commands, &asset_server, compound_room);
            println!("L-shaped room generated");
        }
        2 => {
            // T 形房間 (25% 機率)
            let compound_room = generate_t_shape_room(&mut rng);
            spawn_compound_room(&mut commands, &asset_server, compound_room);
            println!("T-shaped room generated");
        }
        3 => {
            // 十字形房間 (25% 機率)
            let compound_room = generate_cross_shape_room(&mut rng);
            spawn_compound_room(&mut commands, &asset_server, compound_room);
            println!("Cross-shaped room generated");
        }
        _ => {}
    }
}

fn generate_room_tiles(
    commands: &mut Commands,
    asset_server: &AssetServer,
    width: usize,
    height: usize,
    start_x: i32,
    start_y: i32,
    should_generate_door: bool,
) {
    let room_assets = RoomAssets::load_all(asset_server);
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let total_height = height + 1; // 額外一列用於南牆外側

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
            ));

            if matches!(tile_type, RoomTileType::DoorClosed | RoomTileType::DoorOpen) {
                entity_commands.insert(Door {
                    is_open: tile_type == RoomTileType::DoorOpen,
                });
            }
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

fn spawn_compound_room(
    commands: &mut Commands,
    asset_server: &AssetServer,
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
            asset_server,
            rect.width,
            rect.height,
            rect.x,
            rect.y,
            should_generate_door,
        );
    }

    // 生成連接走廊
    generate_corridors(commands, asset_server, &compound_room);

    // 生成複合房間實體
    commands.spawn(compound_room);
}

fn generate_corridors(
    commands: &mut Commands,
    asset_server: &AssetServer,
    compound_room: &CompoundRoom,
) {
    let room_assets = RoomAssets::load_all(asset_server);
    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;

    match compound_room.room_type {
        CompoundRoomType::LShape => {
            let main_rect = &compound_room.rectangles[0];
            let ext_rect = &compound_room.rectangles[1];
            // 在重疊區域以更高 Z 值鋪地板覆蓋牆
            let corridor_x = main_rect.x + main_rect.width as i32 - 1;
            let corridor_y = ext_rect.y + 1;
            commands.spawn((
                Sprite::from_image(room_assets.floor_indoor.clone()),
                Transform::from_translation(Vec3::new(
                    corridor_x as f32 * tile_size,
                    corridor_y as f32 * tile_size,
                    Z_LAYER_GRID + 0.2,
                ))
                .with_scale(Vec3::splat(PLAYER_SCALE)),
                RoomTile {
                    tile_type: RoomTileType::Floor,
                },
            ));
        }
        CompoundRoomType::TShape => {
            let top_beam = &compound_room.rectangles[0];
            let bottom_pillar = &compound_room.rectangles[1];
            // T型連接點在豎梁與橫梁的交界處
            let overlap_x_start = (top_beam.x).max(bottom_pillar.x);
            let overlap_x_end = (top_beam.x + top_beam.width as i32)
                .min(bottom_pillar.x + bottom_pillar.width as i32);
            let overlap_y = top_beam.y;

            for x in overlap_x_start..overlap_x_end {
                commands.spawn((
                    Sprite::from_image(room_assets.floor_indoor.clone()),
                    Transform::from_translation(Vec3::new(
                        x as f32 * tile_size,
                        overlap_y as f32 * tile_size,
                        Z_LAYER_GRID + 0.2,
                    ))
                    .with_scale(Vec3::splat(PLAYER_SCALE)),
                    RoomTile {
                        tile_type: RoomTileType::Floor,
                    },
                ));
            }
        }
        CompoundRoomType::Cross => {
            let h_beam = &compound_room.rectangles[0];
            let v_beam = &compound_room.rectangles[1];
            // 十字型連接點在兩橫梁的中心交叉處
            let overlap_x_start = (h_beam.x).max(v_beam.x);
            let overlap_x_end =
                (h_beam.x + h_beam.width as i32).min(v_beam.x + v_beam.width as i32);
            let overlap_y_start = (h_beam.y).max(v_beam.y);
            let overlap_y_end =
                (h_beam.y + h_beam.height as i32).min(v_beam.y + v_beam.height as i32);

            for x in overlap_x_start..overlap_x_end {
                for y in overlap_y_start..overlap_y_end {
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
                    ));
                }
            }
        }
    }
}
