use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::widget::ImageNode;
use bevy::ui::{AlignItems, BorderColor, Display, FlexDirection, Node, PositionType, UiRect, Val};

#[derive(Component)]
pub struct PlayerStatsTitleText;

#[derive(Component)]
pub struct PlayerStatsLevelText;

#[derive(Component)]
pub struct PlayerStatsAttackText;

#[derive(Component)]
pub struct PlayerStatsDefenseText;

#[derive(Component)]
pub struct PlayerStatsStaminaText;

#[derive(Component)]
pub struct PlayerStatsStatusRow;

pub fn spawn_player_stats_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(PLAYER_STATS_FONT_PATH);
    let sword_icon = asset_server.load(PLAYER_STATS_ATTACK_ICON_PATH);
    let shield_icon = asset_server.load(PLAYER_STATS_DEFENSE_ICON_PATH);
    let skull_icon = asset_server.load(PLAYER_STATS_POISON_ICON_PATH);

    commands
        .spawn((
            PlayerStatsPanel,
            Node {
                width: Val::Px(PLAYER_STATS_PANEL_WIDTH),
                min_height: Val::Px(PLAYER_STATS_PANEL_HEIGHT),
                position_type: PositionType::Absolute,
                top: Val::Px(PLAYER_STATS_PANEL_TOP_OFFSET),
                left: Val::Px(PLAYER_STATS_PANEL_LEFT_OFFSET),
                padding: UiRect {
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                    top: Val::Px(12.0),
                    bottom: Val::Px(12.0),
                },
                border: UiRect::all(Val::Px(1.0)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.92)),
            BorderColor(Color::srgba(0.15, 0.15, 0.2, 0.65)),
            Name::new("PlayerStatsPanel"),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerStatsTitleText,
                Text::new("PLAYER"),
                TextFont {
                    font: font.clone(),
                    font_size: PLAYER_STATS_FONT_SIZE * 1.1,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.93, 0.9)),
            ));

            parent.spawn((
                PlayerStatsLevelText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: PLAYER_STATS_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.85, 0.83, 0.78)),
            ));

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..Default::default()
                    },
                    Name::new("PlayerStatsAttackRow"),
                ))
                .with_children(|row| {
                    row.spawn((
                        ImageNode::new(sword_icon.clone()),
                        Node {
                            width: Val::Px(PLAYER_STATS_ICON_SIZE),
                            height: Val::Px(PLAYER_STATS_ICON_SIZE),
                            ..Default::default()
                        },
                        Name::new("PlayerStatsAttackIcon"),
                    ));

                    row.spawn((
                        PlayerStatsAttackText,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: PLAYER_STATS_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.95, 0.93, 0.9)),
                    ));
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..Default::default()
                    },
                    Name::new("PlayerStatsDefenseRow"),
                ))
                .with_children(|row| {
                    row.spawn((
                        ImageNode::new(shield_icon.clone()),
                        Node {
                            width: Val::Px(PLAYER_STATS_ICON_SIZE),
                            height: Val::Px(PLAYER_STATS_ICON_SIZE),
                            ..Default::default()
                        },
                        Name::new("PlayerStatsDefenseIcon"),
                    ));

                    row.spawn((
                        PlayerStatsDefenseText,
                        Text::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: PLAYER_STATS_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.95, 0.93, 0.9)),
                    ));
                });

            parent.spawn((
                PlayerStatsStaminaText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: PLAYER_STATS_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.85, 0.83, 0.78)),
            ));

            parent
                .spawn((
                    PlayerStatsStatusRow,
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::None,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..Default::default()
                    },
                    Name::new("PlayerStatsStatusRow"),
                ))
                .with_children(|row| {
                    row.spawn((
                        ImageNode::new(skull_icon),
                        Node {
                            width: Val::Px(PLAYER_STATS_ICON_SIZE),
                            height: Val::Px(PLAYER_STATS_ICON_SIZE),
                            ..Default::default()
                        },
                        Name::new("PlayerStatsStatusIcon"),
                    ));
                });
        });
}

pub fn update_player_stats_panel(
    player_query: Query<
        (
            &Attack,
            &Defense,
            &Stamina,
            Option<&Poisoned>,
            &PlayerProgression,
        ),
        With<Player>,
    >,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<PlayerStatsLevelText>>,
        Query<&mut Text, With<PlayerStatsAttackText>>,
        Query<&mut Text, With<PlayerStatsDefenseText>>,
        Query<&mut Text, With<PlayerStatsStaminaText>>,
    )>,
    mut status_query: Query<&mut Node, With<PlayerStatsStatusRow>>,
) {
    let Some((attack, defense, stamina, poison_state, progression)) = player_query.iter().next()
    else {
        return;
    };

    if let Some(mut level_text) = text_queries.p0().iter_mut().next() {
        let content = if let Some(requirement) = progression.next_level_requirement() {
            format!(
                "LV {:>2}   EXP {:>4}/{:>4}",
                progression.level, progression.experience, requirement
            )
        } else {
            format!("LV {:>2}   EXP MAX", progression.level)
        };
        *level_text = Text::new(content);
    }

    if let Some(mut attack_text) = text_queries.p1().iter_mut().next() {
        let value = format!(
            "{:>3}  (base {:>3} | bonus {:+3} | x{:.1})",
            attack.value(),
            attack.base,
            attack.bonus,
            attack.multiplier
        );
        *attack_text = Text::new(value);
    }

    if let Some(mut defense_text) = text_queries.p2().iter_mut().next() {
        let value = format!(
            "{:>3}  (base {:>3} | bonus {:+3} | x{:.1})",
            defense.value(),
            defense.base,
            defense.bonus,
            defense.multiplier
        );
        *defense_text = Text::new(value);
    }

    if let Some(mut stamina_text) = text_queries.p3().iter_mut().next() {
        let value = format!("regen {:>4.1}/s", stamina.regen_per_second);
        *stamina_text = Text::new(value);
    }

    if let Some(mut status_row) = status_query.iter_mut().next() {
        status_row.display = if poison_state.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
}
