use crate::resources::{environment_assets::EnvironmentAssets, room_assets::RoomAssets};
use crate::systems::{movement_system, world::*};
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                initialize_room_assets,
                initialize_environment_assets,
                spawn_world_floor_and_bounds,
            )
                .chain(),
        )
        .add_systems(Update, enforce_world_bounds_system.after(movement_system));
    }
}

fn initialize_room_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let room_assets = RoomAssets::load_all(&asset_server);
    commands.insert_resource(room_assets);
}

fn initialize_environment_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let environment_assets = EnvironmentAssets::load_all(&asset_server);
    commands.insert_resource(environment_assets);
}
