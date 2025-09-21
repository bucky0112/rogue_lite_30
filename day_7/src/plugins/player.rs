use crate::systems::*;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(PostStartup, spawn_player)
            .add_systems(Update, (movement_system, health_system));
    }
}
