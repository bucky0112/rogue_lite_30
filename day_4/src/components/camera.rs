use bevy::prelude::*;

#[derive(Component)]
pub struct CameraFollow {
    pub speed: f32,
}

impl CameraFollow {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}

