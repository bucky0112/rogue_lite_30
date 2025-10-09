use crate::resources::{LevelBuildContext, LevelExitAssets, LevelState};
use crate::systems::level::{
    finalize_level_load, handle_level_requests, process_level_layout, schedule_initial_level,
};
use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelState>()
            .init_resource::<LevelBuildContext>()
            .add_event::<crate::systems::level::LevelAdvanceRequestEvent>()
            .add_event::<crate::systems::level::LevelLoadedEvent>()
            .add_systems(Startup, initialize_level_exit_assets)
            .add_systems(PostStartup, schedule_initial_level)
            .add_systems(Update, handle_level_requests)
            .add_systems(Update, process_level_layout.after(handle_level_requests))
            .add_systems(PostUpdate, finalize_level_load);
    }
}

fn initialize_level_exit_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = LevelExitAssets::load(&asset_server);
    commands.insert_resource(assets);
}
