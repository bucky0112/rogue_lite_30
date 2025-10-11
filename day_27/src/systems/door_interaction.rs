use crate::components::player::{Player, PlayerDead};
use crate::components::world::{Door, RoomTile, RoomTileType};
use crate::constants::*;
use crate::resources::room_assets::RoomAssets;
use bevy::prelude::*;

#[derive(Event)]
pub struct DoorInteractionEvent;

#[derive(Event, Clone, Copy)]
pub struct DoorStateChangedEvent {
    pub is_open: bool,
}

/// 門交互系統 - 處理玩家與門的交互
pub fn door_interaction_system(
    mut door_query: Query<
        (Entity, &mut Door, &mut RoomTile, &Transform, &mut Sprite),
        Without<Player>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<PlayerDead>)>,
    mut door_events: EventReader<DoorInteractionEvent>,
    room_assets: Res<RoomAssets>,
    mut door_state_events: EventWriter<DoorStateChangedEvent>,
) {
    let player_transform = match player_query.single() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    // 只有在收到門交互事件時才處理
    for _event in door_events.read() {
        let mut closest_door = None;
        let mut closest_distance = f32::INFINITY;

        // 找到最近的門
        for (entity, door, room_tile, door_transform, sprite) in door_query.iter() {
            let distance = player_transform
                .translation
                .distance(door_transform.translation);
            let interaction_distance = ROOM_TILE_SIZE * PLAYER_SCALE * 10.0; // 10個瓷磚的距離

            if distance <= interaction_distance && distance < closest_distance {
                closest_distance = distance;
                closest_door = Some((entity, door, room_tile, door_transform, sprite));
            }
        }

        // 如果找到最近的門，切換其狀態
        if let Some((entity, _door, _room_tile, _door_transform, _sprite)) = closest_door {
            if let Ok((_, mut door, mut room_tile, _, mut sprite)) = door_query.get_mut(entity) {
                // 切換門的狀態
                door.is_open = !door.is_open;

                // 更新視覺效果
                if door.is_open {
                    room_tile.tile_type = RoomTileType::DoorOpen;
                    sprite.image = room_assets.door_open.clone();
                    info!("🚪 Door opened; the player can pass through");
                } else {
                    room_tile.tile_type = RoomTileType::DoorClosed;
                    sprite.image = room_assets.door_closed.clone();
                    info!("🚪 Door closed; the path is blocked");
                }

                door_state_events.write(DoorStateChangedEvent {
                    is_open: door.is_open,
                });
            }
        }
    }
}
