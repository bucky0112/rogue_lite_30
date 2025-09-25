use crate::resources::room_assets::RoomAssets;
use crate::systems::world::*;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (initialize_room_assets, spawn_room).chain());
    }
}

fn initialize_room_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let room_assets = RoomAssets::load_all(&asset_server);
    commands.insert_resource(room_assets);
}
