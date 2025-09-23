use crate::components::player::Player;
use crate::components::world::{Door, RoomTile};
use crate::constants::*;
use crate::systems::attack::AttackInputEvent;
use crate::systems::door_interaction::DoorInteractionEvent;
use bevy::prelude::*;

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    door_query: Query<(&Door, &Transform), (With<RoomTile>, Without<Player>)>,
    mut door_events: EventWriter<DoorInteractionEvent>,
    mut attack_events: EventWriter<AttackInputEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let player_transform = match player_query.single() {
            Ok(transform) => transform,
            Err(_) => return,
        };

        let interaction_distance = ROOM_TILE_SIZE * PLAYER_SCALE * 10.0; // 10個瓷磚距離
        let mut near_door = false;

        for (_door, door_transform) in &door_query {
            let distance = player_transform
                .translation
                .distance(door_transform.translation);
            if distance <= interaction_distance {
                near_door = true;
                break;
            }
        }

        if near_door {
            door_events.write(DoorInteractionEvent);
            info!("門交互事件已發送！");
        } else {
            attack_events.write(AttackInputEvent);
        }
    }
}
