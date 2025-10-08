use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct LevelExitAssets {
    pub left_panel: Handle<Image>,
    pub right_panel: Handle<Image>,
}

impl LevelExitAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            left_panel: asset_server.load("doors/next_left.png"),
            right_panel: asset_server.load("doors/next_right.png"),
        }
    }
}
