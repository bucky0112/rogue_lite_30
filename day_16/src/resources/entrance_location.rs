use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct EntranceLocation {
    pub position: Vec3,
}

impl EntranceLocation {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}
