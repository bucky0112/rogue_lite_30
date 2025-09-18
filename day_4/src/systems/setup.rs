use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, CameraFollow::new(CAMERA_FOLLOW_SPEED),));
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("characters/knight_lv1.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(PLAYER_SCALE)),
        Health::new(PLAYER_INITIAL_HEALTH),
        Velocity::zero(),
    ));
    info!("騎士玩家已誕生！");
}

