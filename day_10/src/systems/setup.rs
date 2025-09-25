use crate::components::*;
use crate::constants::*;
use crate::resources::EntranceLocation;
use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, CameraFollow::new(CAMERA_FOLLOW_SPEED)));
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    entrance_location: Option<Res<EntranceLocation>>,
) {
    let spawn_position = entrance_location
        .map(|location| location.position)
        .unwrap_or_else(|| Vec3::new(0.0, -ROOM_TILE_SIZE * PLAYER_SCALE * 3.0, 10.0));

    let player_entity = commands
        .spawn((
            Player,
            Sprite::from_image(asset_server.load("characters/players/knight_lv1.png")),
            Transform::from_translation(spawn_position) // 更高的Z值，確保在房間瓷磚之上
                .with_scale(Vec3::splat(PLAYER_SCALE)),
            Health::new(PLAYER_INITIAL_HEALTH),
            Velocity::zero(),
            PlayerFacing::new(),
            InputVector(Vec2::ZERO),
        ))
        .id();

    // Spawn weapon as child of player
    let weapon_entity = commands
        .spawn((
            Weapon,
            Sprite::from_image(asset_server.load("weapons/sword.png")),
            Transform::from_translation(Vec3::new(8.0, 2.0, 1.0)) // 相對於玩家的位置
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
        ))
        .id();

    // Set weapon as child of player
    commands
        .entity(player_entity)
        .add_children(&[weapon_entity]);

    info!("騎士玩家與武器已誕生！");
}
