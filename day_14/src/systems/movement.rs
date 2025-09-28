use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;

pub fn movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut PlayerFacing,
            &mut InputVector,
        ),
        (With<Player>, Without<PlayerDead>),
    >,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, mut facing, mut input_vector) in &mut query {
        velocity.x = 0.0;
        velocity.y = 0.0;

        let mut raw_input = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.y = PLAYER_SPEED;
            raw_input.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.y = -PLAYER_SPEED;
            raw_input.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            velocity.x = -PLAYER_SPEED;
            raw_input.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            velocity.x = PLAYER_SPEED;
            raw_input.x += 1.0;
        }

        // Update input vector for room transition system
        input_vector.0 = if raw_input.length() > INPUT_DEADZONE {
            raw_input.normalize()
        } else {
            Vec2::ZERO
        };

        // Update facing direction if moving
        if velocity.x != 0.0 || velocity.y != 0.0 {
            facing.direction = Vec2::new(velocity.x, velocity.y).normalize();
        }

        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}
