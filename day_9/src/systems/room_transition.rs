use crate::components::player::{InputVector, Player, Velocity};
use crate::components::world::Door;
use crate::constants::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct TransitionCooldown {
    pub timer: Timer,
}

impl Default for TransitionCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

/// 房間切換系統 - 基於玩家移動方向的房間切換機制
pub fn room_transition_system(
    door_query: Query<(&Door, &Transform), Without<Player>>,
    mut player_query: Query<(&mut Transform, &Velocity, &InputVector), With<Player>>,
    mut transition_cooldown: ResMut<TransitionCooldown>,
    time: Res<Time>,
) {
    // 更新冷卻計時器
    transition_cooldown.timer.tick(time.delta());

    // 如果還在冷卻中，不執行傳送
    if !transition_cooldown.timer.finished() {
        return;
    }

    let (mut player_transform, velocity, input_vector) = match player_query.single_mut() {
        Ok(result) => result,
        Err(_) => return,
    };

    let tile_size = ROOM_TILE_SIZE * PLAYER_SCALE;
    let trigger_distance = tile_size * 3.0; // 3個瓷磚距離作為觸發範圍
    let teleport_offset = tile_size * 1.5; // 根據瓷磚大小計算傳送距離，適用不同縮放

    // 嘗試使用輸入向量，若沒有即退而求其次使用當前速度來判斷運動方向
    let mut movement = input_vector.0;
    if movement.length_squared() <= f32::EPSILON {
        movement = Vec2::new(velocity.x, velocity.y);
        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }
    }

    for (door, door_transform) in &door_query {
        let door_pos = door_transform.translation.truncate();
        let player_pos = player_transform.translation.truncate();
        let distance = door_pos.distance(player_pos);

        // 只有在玩家靠近門且門是開啟的情況下才檢查
        if door.is_open && distance < trigger_distance {
            let door_to_player = player_pos - door_pos;

            // 檢查玩家是否在向門的方向移動
            let movement_threshold = 0.1;
            let moving_up = movement.y > movement_threshold;
            let moving_down = movement.y < -movement_threshold;

            // 房間切換邏輯：
            // 1. 如果玩家在門下方且向上移動 -> 傳送到門上方（進入房間）
            // 2. 如果玩家在門上方且向下移動 -> 傳送到門下方（離開房間）

            if door_to_player.y < -20.0 && moving_up {
                // 玩家在門下方，向上移動 - 進入房間
                let new_position = door_pos + Vec2::new(0.0, teleport_offset);
                player_transform.translation.x = new_position.x;
                player_transform.translation.y = new_position.y;

                transition_cooldown.timer.reset();
                info!(
                    "✅ 玩家進入房間！從 {:?} 傳送到 {:?}",
                    player_pos, new_position
                );
            } else if door_to_player.y > 20.0 && moving_down {
                // 玩家在門上方，向下移動 - 離開房間
                let new_position = door_pos + Vec2::new(0.0, -teleport_offset);
                player_transform.translation.x = new_position.x;
                player_transform.translation.y = new_position.y;

                transition_cooldown.timer.reset();
                info!(
                    "✅ 玩家離開房間！從 {:?} 傳送到 {:?}",
                    player_pos, new_position
                );
            }
        }
    }
}
