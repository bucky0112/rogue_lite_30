use crate::components::player::{Player, PlayerFacing};
use crate::components::world::{Door, RoomTile};
use crate::constants::*;
use crate::systems::attack::AttackInputEvent;
use crate::systems::door_interaction::DoorInteractionEvent;
use bevy::prelude::*;

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &PlayerFacing), With<Player>>,
    door_query: Query<(&Door, &Transform), (With<RoomTile>, Without<Player>)>,
    mut door_events: EventWriter<DoorInteractionEvent>,
    mut attack_events: EventWriter<AttackInputEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let Ok((player_transform, facing)) = player_query.single() else {
            return;
        };

        let player_position = player_transform.translation.truncate();
        let facing_direction = facing.direction.normalize_or_zero();

        let mut near_door = false;

        for (_door, door_transform) in &door_query {
            let to_door = door_transform.translation.truncate() - player_position;
            let distance = to_door.length();

            if distance == 0.0 || distance > DOOR_INTERACTION_RADIUS {
                continue;
            }

            if facing_direction != Vec2::ZERO {
                let direction = to_door / distance;
                if facing_direction.dot(direction) < DOOR_INTERACTION_FACING_COS_THRESHOLD {
                    continue;
                }
            }

            near_door = true;
            break;
        }

        if near_door {
            door_events.write(DoorInteractionEvent);
            info!("門交互事件已發送！");
        } else {
            attack_events.write(AttackInputEvent);
        }
    }
}
