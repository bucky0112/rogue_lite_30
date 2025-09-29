use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{AlignItems, BorderColor, Display, JustifyContent, Node, PositionType, UiRect, Val};

pub fn spawn_player_stats_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                    top: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                },
                border: UiRect::all(Val::Px(1.0)),
                display: Display::Flex,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.92)),
            BorderColor(Color::srgba(0.15, 0.15, 0.2, 0.65)),
            Name::new("PlayerStatsPanel"),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerStatsText,
                Text::new(""),
                TextFont {
                    font: asset_server.load(PLAYER_STATS_FONT_PATH),
                    font_size: PLAYER_STATS_FONT_SIZE,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.93, 0.9)),
            ));
        });
}

pub fn update_player_stats_panel(
    player_query: Query<(&Attack, &Defense, &Stamina, Option<&Poisoned>), With<Player>>,
    mut text_query: Query<&mut Text, With<PlayerStatsText>>,
) {
    let Some((attack, defense, stamina, poison_state)) = player_query.iter().next() else {
        return;
    };

    let Some(mut text) = text_query.iter_mut().next() else {
        return;
    };

    let attack_value = attack.value();
    let defense_value = defense.value();

    let header_line = "PLAYER STATS";
    let attack_line = format!(
        "ATK  {:>3}  (base {:>3}, bonus {:+3}, x{:.1})",
        attack_value, attack.base, attack.bonus, attack.multiplier,
    );

    let defense_line = format!(
        "DEF  {:>3}  (base {:>3}, bonus {:+3}, x{:.1})",
        defense_value, defense.base, defense.bonus, defense.multiplier,
    );

    let stamina_line = format!(
        "STA  {:>5.1}/{:>5.1}  (regen {:>4.1}/s, cost {:>4.0})",
        stamina.current, stamina.max, stamina.regen_per_second, PLAYER_ATTACK_STAMINA_COST,
    );

    let status_line = if poison_state.is_some() {
        "STATUS  POISONED (HP -3 per tick)"
    } else {
        "STATUS  HEALTHY"
    };

    let hint_line_top = "Adjust: [1] ATK +5  [2] ATK -5  [3] DEF +3  [4] DEF -3";
    let hint_line_bottom =
        "        [Q] ATK x-0.1  [W] ATK x+0.1  [A] DEF x-0.1  [S] DEF x+0.1  [R] Reset";
    let hint_line_status = "        [T] REFILL STA  [G] ANTIDOTE  [Y] APPLY TEST POISON";

    let content = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        header_line,
        attack_line,
        defense_line,
        stamina_line,
        status_line,
        hint_line_top,
        hint_line_bottom,
    );

    let content = format!("{}\n{}", content, hint_line_status);

    *text = Text::new(content);
}

pub fn player_attribute_debug_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Attack, &mut Defense), (With<Player>, Without<PlayerDead>)>,
) {
    let Some((mut attack, mut defense)) = query.iter_mut().next() else {
        return;
    };

    let mut changed = false;

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        attack.adjust_bonus(5);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Digit2) {
        attack.adjust_bonus(-5);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Digit3) {
        defense.adjust_bonus(3);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Digit4) {
        defense.adjust_bonus(-3);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        attack.adjust_multiplier(-0.1);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyW) {
        attack.adjust_multiplier(0.1);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        defense.adjust_multiplier(-0.1);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        defense.adjust_multiplier(0.1);
        changed = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        attack.reset_modifiers();
        defense.reset_modifiers();
        changed = true;
    }

    if changed {
        info!(
            "玩家屬性更新：攻擊 {} (加成 {} 倍率 {:.1})，防禦 {} (加成 {} 倍率 {:.1})",
            attack.value(),
            attack.bonus,
            attack.multiplier,
            defense.value(),
            defense.bonus,
            defense.multiplier,
        );
    }
}
