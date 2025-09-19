use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, CameraFollow::new(CAMERA_FOLLOW_SPEED),));
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_entity = commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("characters/knight_lv1.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(PLAYER_SCALE)),
        Health::new(PLAYER_INITIAL_HEALTH),
        Velocity::zero(),
        PlayerFacing::new(),
    )).id();

    // Spawn weapon as child of player
    let weapon_entity = commands.spawn((
        Weapon,
        Sprite::from_image(asset_server.load("weapons/sword.png")),
        Transform::from_translation(Vec3::new(8.0, 2.0, 1.0))
            .with_scale(Vec3::splat(WEAPON_SCALE)),
        WeaponSprites {
            right_sprite: asset_server.load("weapons/sword.png"),
            left_sprite: asset_server.load("weapons/sword_left.png"),
        },
        WeaponOffset {
            base_angle: 0.0,
            position: Vec2::new(8.0, 2.0),
        },
        WeaponSwing {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            from_angle: 0.0,
            to_angle: 0.0,
        },
    )).id();

    // Set weapon as child of player
    commands.entity(player_entity).add_children(&[weapon_entity]);
    
    info!("玩家與武器已誕生！");
}

