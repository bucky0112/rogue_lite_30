use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct EnvironmentAssets {
    pub tree: Handle<Image>,
    pub rock: Handle<Image>,
    pub crate_prop: Handle<Image>,
}

impl EnvironmentAssets {
    pub fn load_all(asset_server: &AssetServer) -> Self {
        Self {
            tree: asset_server.load("environment/tree.png"),
            rock: asset_server.load("environment/rock.png"),
            crate_prop: asset_server.load("environment/rock_floor.png"),
        }
    }
}
