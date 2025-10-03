use bevy::prelude::*;

#[derive(Resource)]
pub struct RoomAssets {
    pub floor_indoor: Handle<Image>,
    pub floor_outdoor: Handle<Image>,
    pub wall_n_inner_corner_w: Handle<Image>,
    pub wall_n_inner_mid: Handle<Image>,
    pub wall_n_inner_corner_e: Handle<Image>,
    pub wall_s_inner_cap_l: Handle<Image>,
    pub wall_s_inner_mid: Handle<Image>,
    pub wall_s_inner_cap_r: Handle<Image>,
    pub wall_s_outer_cap_l: Handle<Image>,
    pub wall_s_outer_mid: Handle<Image>,
    pub wall_s_outer_cap_r: Handle<Image>,
    pub wall_e_side: Handle<Image>,
    pub wall_w_side: Handle<Image>,
    pub door_closed: Handle<Image>,
    pub door_open: Handle<Image>,
}

impl RoomAssets {
    pub fn load_all(asset_server: &AssetServer) -> Self {
        Self {
            floor_indoor: asset_server.load("floors/floor_indoor.png"),
            floor_outdoor: asset_server.load("floors/floor_outdoor.png"),
            wall_n_inner_corner_w: asset_server.load("walls/wall_N_inner_corner_W.png"),
            wall_n_inner_mid: asset_server.load("walls/wall_N_inner_mid.png"),
            wall_n_inner_corner_e: asset_server.load("walls/wall_N_inner_corner_E.png"),
            wall_s_inner_cap_l: asset_server.load("walls/wall_S_inner_cap_L.png"),
            wall_s_inner_mid: asset_server.load("walls/wall_S_inner_mid.png"),
            wall_s_inner_cap_r: asset_server.load("walls/wall_S_inner_cap_R.png"),
            wall_s_outer_cap_l: asset_server.load("walls/wall_S_outer_cap_L.png"),
            wall_s_outer_mid: asset_server.load("walls/wall_S_outer_mid.png"),
            wall_s_outer_cap_r: asset_server.load("walls/wall_S_outer_cap_R.png"),
            wall_e_side: asset_server.load("walls/wall_E_side.png"),
            wall_w_side: asset_server.load("walls/wall_W_side.png"),
            door_closed: asset_server.load("doors/door_closed.png"),
            door_open: asset_server.load("doors/door_open.png"),
        }
    }
}
