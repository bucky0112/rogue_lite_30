use crate::components::enemy::Enemy;
use crate::components::player::{Player, PlayerDead};
use crate::components::world::{Door, EnvironmentProp, RoomTile, RoomTileType};
use crate::constants::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// 牆壁碰撞檢測系統 - 阻止玩家穿牆
pub fn wall_collision_system(
    wall_query: Query<(Entity, &RoomTile, &Transform), Without<Player>>,
    door_query: Query<&Door>,
    environment_query: Query<(&EnvironmentProp, &Transform), Without<Player>>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerDead>)>,
) {
    let mut player_transform = match player_query.single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE; // 64 像素
    let collision_threshold = tile_size * 0.7; // 約44.8像素的碰撞檢測範圍

    let player_pos = player_transform.translation.truncate();

    // 建立位置到瓷磚的映射，優先考慮地板瓷磚
    let mut position_tiles: HashMap<(i32, i32), (Entity, &RoomTile, &Transform)> = HashMap::new();

    for (entity, room_tile, wall_transform) in &wall_query {
        let pos_key = (
            (wall_transform.translation.x / tile_size).round() as i32,
            (wall_transform.translation.y / tile_size).round() as i32,
        );

        // 如果這個位置已經有瓷磚了
        if let Some((existing_entity, existing_tile, existing_transform)) =
            position_tiles.get(&pos_key)
        {
            let existing_priority = tile_priority(&existing_tile.tile_type);
            let new_priority = tile_priority(&room_tile.tile_type);
            if new_priority > existing_priority {
                position_tiles.insert(pos_key, (entity, room_tile, wall_transform));
            } else if new_priority == existing_priority && entity.index() < existing_entity.index()
            {
                // 穩定排序：若優先級相同，保留較小 entity index，避免跳動
                position_tiles.insert(pos_key, (entity, room_tile, wall_transform));
            } else {
                // 保留現有資料
                let _ = existing_entity;
                let _ = existing_transform;
            }
        } else {
            position_tiles.insert(pos_key, (entity, room_tile, wall_transform));
        }
    }

    // 檢查碰撞，使用優先級處理後的瓷磚
    for (_pos_key, (entity, room_tile, wall_transform)) in position_tiles {
        if is_wall_tile(&room_tile.tile_type) {
            // 對於門，需要檢查是否開啟
            if matches!(
                room_tile.tile_type,
                RoomTileType::DoorClosed | RoomTileType::DoorOpen
            ) {
                if let Ok(door) = door_query.get(entity) {
                    if door.is_open {
                        continue; // 開啟的門不阻擋玩家
                    }
                }
            }

            let wall_pos = wall_transform.translation.truncate();
            let distance = wall_pos.distance(player_pos);

            // 如果玩家太靠近牆壁
            if distance < collision_threshold {
                // 計算推開玩家的方向
                let push_direction = (player_pos - wall_pos).normalize_or_zero();

                // 將玩家推到安全距離
                let safe_distance = collision_threshold + 1.0;
                let new_position = wall_pos + push_direction * safe_distance;

                player_transform.translation.x = new_position.x;
                player_transform.translation.y = new_position.y;

                return; // 一次只處理一個碰撞
            }
        }
    }

    let mut player_pos = player_transform.translation.truncate();
    for (prop, transform) in &environment_query {
        if !prop.blocks_movement {
            continue;
        }

        let prop_pos = transform.translation.truncate();
        let collision_threshold = ENVIRONMENT_PROP_COLLISION_RADIUS;
        let distance = prop_pos.distance(player_pos);

        if distance < collision_threshold {
            let push_direction = (player_pos - prop_pos).normalize_or_zero();
            if push_direction == Vec2::ZERO {
                continue;
            }

            let safe_distance = collision_threshold + 1.0;
            let new_position = prop_pos + push_direction * safe_distance;
            player_transform.translation.x = new_position.x;
            player_transform.translation.y = new_position.y;
            player_pos = player_transform.translation.truncate();
        }
    }
}

/// 判斷是否為牆壁瓷磚類型
fn is_wall_tile(tile_type: &RoomTileType) -> bool {
    match tile_type {
        RoomTileType::Floor => false,
        RoomTileType::DoorOpen => false, // 開啟的門不阻擋
        RoomTileType::FloorOutdoor => false,
        RoomTileType::DoorClosed
        | RoomTileType::WallNInnerCornerW
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

fn tile_priority(tile_type: &RoomTileType) -> u8 {
    match tile_type {
        RoomTileType::FloorOutdoor => 0,
        RoomTileType::DoorOpen => 2,
        RoomTileType::Floor => 4,
        RoomTileType::DoorClosed
        | RoomTileType::WallNInnerCornerW
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

pub fn enemy_wall_collision_system(
    wall_query: Query<(Entity, &RoomTile, &Transform), Without<Enemy>>,
    door_query: Query<&Door>,
    environment_query: Query<(&EnvironmentProp, &Transform), Without<Enemy>>,
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
) {
    let mut position_tiles: HashMap<(i32, i32), (Entity, RoomTileType, Vec3)> = HashMap::new();

    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;

    for (entity, room_tile, wall_transform) in &wall_query {
        let pos_key = (
            (wall_transform.translation.x / tile_size).round() as i32,
            (wall_transform.translation.y / tile_size).round() as i32,
        );

        if let Some((existing_entity, existing_type, existing_translation)) =
            position_tiles.get(&pos_key).cloned()
        {
            let existing_priority = tile_priority(&existing_type);
            let new_priority = tile_priority(&room_tile.tile_type);
            if new_priority > existing_priority
                || (new_priority == existing_priority && entity.index() < existing_entity.index())
            {
                position_tiles.insert(
                    pos_key,
                    (entity, room_tile.tile_type, wall_transform.translation),
                );
            } else {
                let _ = existing_translation;
            }
        } else {
            position_tiles.insert(
                pos_key,
                (entity, room_tile.tile_type, wall_transform.translation),
            );
        }
    }

    for mut enemy_transform in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();
        let scale = enemy_transform.scale.x.max(1.0);
        let collision_threshold = ROOM_TILE_SIZE * scale * 0.7;

        for (entity, tile_type, wall_translation) in position_tiles.values() {
            if !is_wall_tile(tile_type) {
                continue;
            }

            if matches!(tile_type, RoomTileType::DoorClosed | RoomTileType::DoorOpen) {
                if let Ok(door) = door_query.get(*entity) {
                    if door.is_open {
                        continue;
                    }
                }
            }

            let wall_pos = wall_translation.truncate();
            let distance = wall_pos.distance(enemy_pos);

            if distance < collision_threshold {
                let push_direction = (enemy_pos - wall_pos).normalize_or_zero();

                if push_direction == Vec2::ZERO {
                    continue;
                }

                let safe_distance = collision_threshold + 1.0;
                let new_position = wall_pos + push_direction * safe_distance;

                enemy_transform.translation.x = new_position.x;
                enemy_transform.translation.y = new_position.y;

                break;
            }
        }

        let mut current_pos = enemy_transform.translation.truncate();
        for (prop, transform) in &environment_query {
            if !prop.blocks_movement {
                continue;
            }

            let prop_pos = transform.translation.truncate();
            let collision_threshold = ENVIRONMENT_PROP_COLLISION_RADIUS;
            let distance = prop_pos.distance(current_pos);

            if distance < collision_threshold {
                let push_direction = (current_pos - prop_pos).normalize_or_zero();
                if push_direction == Vec2::ZERO {
                    continue;
                }

                let safe_distance = collision_threshold + 1.0;
                let new_position = prop_pos + push_direction * safe_distance;
                enemy_transform.translation.x = new_position.x;
                enemy_transform.translation.y = new_position.y;
                current_pos = enemy_transform.translation.truncate();
            }
        }
    }
}
