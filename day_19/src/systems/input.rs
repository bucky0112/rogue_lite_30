use crate::components::world::{Door, RoomTile};
use crate::components::{Chest, Player, PlayerDead, PlayerFacing};
use crate::constants::*;
use crate::systems::attack::AttackInputEvent;
use crate::systems::chest::ChestInteractionEvent;
use crate::systems::door_interaction::DoorInteractionEvent;
use bevy::prelude::*;

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &PlayerFacing), (With<Player>, Without<PlayerDead>)>,
    chest_query: Query<(Entity, &Transform, &Chest), Without<Player>>,
    door_query: Query<(&Door, &Transform), (With<RoomTile>, Without<Player>)>,
    mut door_events: EventWriter<DoorInteractionEvent>,
    mut chest_events: EventWriter<ChestInteractionEvent>,
    mut attack_events: EventWriter<AttackInputEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let Ok((player_transform, facing)) = player_query.single() else {
            return;
        };

        let player_position = player_transform.translation.truncate();
        let facing_direction = facing.direction.normalize_or_zero();

        let mut nearest_chest: Option<(Entity, f32)> = None;

        for (entity, chest_transform, chest) in &chest_query {
            if !chest.is_closed() {
                continue;
            }

            let distance = player_position.distance(chest_transform.translation.truncate());
            if distance > CHEST_INTERACTION_RADIUS {
                continue;
            }

            if nearest_chest.map_or(true, |(_, current)| distance < current) {
                nearest_chest = Some((entity, distance));
            }
        }

        if let Some((entity, _)) = nearest_chest {
            chest_events.write(ChestInteractionEvent { chest: entity });
            return;
        }

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
