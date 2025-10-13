use crate::systems::{
    enforce_world_bounds_system,
    wall_collision::{enemy_wall_collision_system, wall_collision_system},
};
use bevy::prelude::*;

pub struct WallCollisionPlugin;

impl Plugin for WallCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                wall_collision_system.after(enforce_world_bounds_system),
                enemy_wall_collision_system.after(enforce_world_bounds_system),
            ),
        );
    }
}
