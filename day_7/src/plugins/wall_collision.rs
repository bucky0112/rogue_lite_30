use crate::systems::wall_collision::wall_collision_system;
use bevy::prelude::*;

pub struct WallCollisionPlugin;

impl Plugin for WallCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, wall_collision_system);
    }
}
