use bevy::prelude::*;
use crate::systems::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow_system);
    }
}

