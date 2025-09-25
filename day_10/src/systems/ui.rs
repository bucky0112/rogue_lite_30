use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui::{BackgroundColor, BorderColor, Node, PositionType, UiRect, Val};

pub fn setup_player_health_ui(mut commands: Commands) {
    commands
        .spawn((
            PlayerHealthUiRoot,
            Node {
                width: Val::Px(PLAYER_HEALTH_BAR_WIDTH),
                height: Val::Px(PLAYER_HEALTH_BAR_HEIGHT),
                position_type: PositionType::Absolute,
                top: Val::Px(PLAYER_HEALTH_BAR_MARGIN),
                left: Val::Px(PLAYER_HEALTH_BAR_MARGIN),
                border: UiRect::all(Val::Px(2.0)),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.9)),
            BorderColor(Color::srgba(0.02, 0.02, 0.02, 1.0)),
            Name::new("PlayerHealthBar"),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerHealthUiFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgba(0.84, 0.16, 0.18, 1.0)),
                Name::new("PlayerHealthFill"),
            ));
        });
}

pub fn update_player_health_ui(
    player_query: Query<&Health, With<Player>>,
    mut fill_query: Query<&mut Node, With<PlayerHealthUiFill>>,
) {
    let Some(health) = player_query.iter().next() else {
        return;
    };

    let Some(mut node) = fill_query.iter_mut().next() else {
        return;
    };

    if health.max <= 0 {
        node.width = Val::Percent(0.0);
        return;
    }

    let ratio = (health.current as f32).clamp(0.0, health.max as f32) / health.max as f32;
    node.width = Val::Percent((ratio * 100.0).clamp(0.0, 100.0));
}

pub fn spawn_enemy_health_bars(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Health), (With<Enemy>, Added<Enemy>)>,
) {
    for (enemy_entity, transform, health) in &query {
        if health.max <= 0 {
            continue;
        }

        let offset = Vec3::new(0.0, ENEMY_HEALTH_BAR_OFFSET_Y, 6.0);
        let base_translation = transform.translation + offset;

        commands
            .spawn((
                EnemyHealthBarRoot,
                HealthBarFollow::new(enemy_entity, offset),
                Transform::from_translation(base_translation),
                Name::new("EnemyHealthBar"),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Sprite::from_color(
                        Color::srgba(0.08, 0.08, 0.08, 0.85),
                        Vec2::new(ENEMY_HEALTH_BAR_WIDTH, ENEMY_HEALTH_BAR_HEIGHT),
                    ),
                    Name::new("EnemyHealthBack"),
                ));

                let mut fill_sprite = Sprite::from_color(
                    Color::srgba(0.86, 0.18, 0.22, 0.95),
                    Vec2::new(ENEMY_HEALTH_BAR_WIDTH, ENEMY_HEALTH_BAR_HEIGHT),
                );
                fill_sprite.anchor = Anchor::CenterLeft;

                parent.spawn((
                    EnemyHealthBarFill,
                    HealthBarTarget::new(enemy_entity),
                    fill_sprite,
                    Transform::from_translation(Vec3::new(-ENEMY_HEALTH_BAR_WIDTH / 2.0, 0.0, 1.0)),
                    Name::new("EnemyHealthFill"),
                ));
            });
    }
}

pub fn update_enemy_health_bar_positions(
    mut commands: Commands,
    owner_query: Query<&GlobalTransform>,
    mut bar_query: Query<(Entity, &HealthBarFollow, &mut Transform), With<EnemyHealthBarRoot>>,
) {
    for (entity, follow, mut transform) in &mut bar_query {
        match owner_query.get(follow.target) {
            Ok(owner_transform) => {
                let mut translation = owner_transform.translation();
                translation += follow.offset;
                transform.translation = translation;
            }
            Err(_) => {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn update_enemy_health_bar_fill(
    health_query: Query<&Health>,
    mut fill_query: Query<(&HealthBarTarget, &mut Transform), With<EnemyHealthBarFill>>,
) {
    for (target, mut transform) in &mut fill_query {
        let Ok(health) = health_query.get(target.target) else {
            continue;
        };

        if health.max <= 0 {
            transform.scale.x = 0.0;
            continue;
        }

        let ratio = (health.current as f32).clamp(0.0, health.max as f32) / health.max as f32;
        transform.scale.x = ratio.clamp(0.0, 1.0);
    }
}
