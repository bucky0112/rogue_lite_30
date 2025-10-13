use crate::resources::CameraShake;
use crate::systems::*;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraShake>().add_systems(
            Update,
            (
                camera_follow_system,
                apply_camera_shake_system
                    .after(camera_follow_system)
                    .after(trigger_camera_shake_on_enemy_hit),
            ),
        );
    }
}
