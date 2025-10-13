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

    let progression = PlayerProgression::new();
    let base_attack = progression.base_attack();
    let base_defense = progression.base_defense();
    let sprite_path = progression.sprite_path();

    let player_entity = commands
        .spawn((
            Player,
            Sprite::from_image(asset_server.load(sprite_path)),
            Transform::from_translation(spawn_position) // 更高的Z值，確保在房間瓷磚之上
                .with_scale(Vec3::splat(PLAYER_SCALE)),
            Health::new(PLAYER_INITIAL_HEALTH),
            Attack::new(base_attack),
            Defense::new(base_defense),
            Stamina::new(PLAYER_MAX_STAMINA, PLAYER_STAMINA_REGEN_PER_SECOND),
            Velocity::zero(),
            PlayerFacing::new(),
            InputVector(Vec2::ZERO),
            EquippedWeapon::new(WeaponKind::Level1),
            progression,
        ))
        .id();

    // Spawn weapon as child of player
    let weapon_entity = commands
        .spawn((
            Weapon,
            Sprite::from_image(asset_server.load(WeaponKind::Level1.right_sprite_path())),
            Transform::from_translation(Vec3::new(
                WEAPON_IDLE_OFFSET_X,
                WEAPON_IDLE_OFFSET_Y,
                WEAPON_Z,
            )) // 相對於玩家的位置
            .with_scale(Vec3::splat(WEAPON_SCALE)),
            WeaponSprites {
                right_sprite: asset_server.load(WeaponKind::Level1.right_sprite_path()),
                left_sprite: asset_server.load(WeaponKind::Level1.left_sprite_path()),
            },
            WeaponOffset {
                base_angle: 0.0,
                position: Vec2::new(WEAPON_IDLE_OFFSET_X, WEAPON_IDLE_OFFSET_Y),
            },
            WeaponSwing {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
                from_angle: 0.0,
                to_angle: 0.0,
            },
            Name::new(format!("Equipped{}", WeaponKind::Level1.display_name())),
        ))
        .id();

    // Set weapon as child of player
    commands
        .entity(player_entity)
        .add_children(&[weapon_entity]);

    commands.spawn((
        AttackReticle::new(),
        Sprite::from_image(asset_server.load(ATTACK_RETICLE_SPRITE_PATH)),
        Transform::from_translation(Vec3::new(
            spawn_position.x + ATTACK_RETICLE_DISTANCE,
            spawn_position.y,
            spawn_position.z + ATTACK_RETICLE_Z_OFFSET,
        ))
        .with_scale(Vec3::splat(PLAYER_SCALE)),
        Name::new("AttackReticle"),
    ));

    dev_info!("Knight player and weapon spawned");
}
