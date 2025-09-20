use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;

pub fn movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut PlayerFacing), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, mut facing) in &mut query {
        velocity.x = 0.0;
        velocity.y = 0.0;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.y = PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.y = -PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            velocity.x = -PLAYER_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            velocity.x = PLAYER_SPEED;
        }

        // Update facing direction if moving
        if velocity.x != 0.0 || velocity.y != 0.0 {
            facing.direction = Vec2::new(velocity.x, velocity.y).normalize();
        }

        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}
