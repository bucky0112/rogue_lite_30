use crate::components::*;
use crate::resources::CameraShake;
use bevy::prelude::*;

pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), (With<CameraFollow>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut camera_transform, camera_follow) in &mut camera_query {
            let target_position = player_transform.translation;
            let current_position = camera_transform.translation;

            let lerp_factor = (camera_follow.speed * time.delta_secs()).min(1.0);

            camera_transform.translation.x =
                current_position.x + (target_position.x - current_position.x) * lerp_factor;
            camera_transform.translation.y =
                current_position.y + (target_position.y - current_position.y) * lerp_factor;
        }
    }
}

pub fn apply_camera_shake_system(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut camera_query: Query<&mut Transform, With<CameraFollow>>,
) {
    let delta = time.delta_secs();
    shake.update(delta);

    let offset = shake.current_offset();
    if offset.length_squared() <= f32::EPSILON {
        return;
    }

    for mut transform in &mut camera_query {
        transform.translation.x += offset.x;
        transform.translation.y += offset.y;
    }
}
