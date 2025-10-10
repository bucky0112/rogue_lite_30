use crate::components::{
    Attack, Defense, EquippedShield, EquippedWeapon, Health, MainMenuAction, MainMenuButton,
    MainMenuRoot, PauseMenuAction, PauseMenuButton, PauseMenuRoot, Player, PlayerDead,
    PlayerProgression, Poisoned,
};
use crate::constants::{
    MENU_BUTTON_FONT_SIZE, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH, MENU_FONT_PATH,
    MENU_OVERLAY_COLOR, MENU_TITLE_FONT_SIZE,
};
use crate::resources::{
    GamePhase, GameSaveData, GameSession, LevelBuildContext, LevelState, PlayerDeathState,
};
use crate::systems::equipment::{ShieldEquipEvent, WeaponEquipEvent};
use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{
    AlignItems, BackgroundColor, BorderColor, FlexDirection, GlobalZIndex, Interaction,
    JustifyContent, Node, PositionType, UiRect, Val,
};
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Event, Debug, Clone, Copy)]
pub struct StartNewGameEvent;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestSaveGameEvent;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestLoadGameEvent {
    pub from_main_menu: bool,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ResumeGameplayEvent;

pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<GameSession>,
) {
    if session.main_menu_root.is_none() {
        let entity = build_main_menu(&mut commands, &asset_server);
        session.main_menu_root = Some(entity);
    }

    session.set_phase(GamePhase::MainMenu);
}

pub fn handle_main_menu_interactions(
    mut interactions: Query<
        (&Interaction, &MainMenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut start_events: EventWriter<StartNewGameEvent>,
    mut load_events: EventWriter<RequestLoadGameEvent>,
) {
    for (interaction, button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => match button.action {
                MainMenuAction::NewGame => {
                    start_events.write(StartNewGameEvent);
                }
                MainMenuAction::LoadGame => {
                    load_events.write(RequestLoadGameEvent {
                        from_main_menu: true,
                    });
                }
            },
            Interaction::Hovered => {
                background.0 = Color::srgba(0.35, 0.28, 0.25, 0.9);
            }
            Interaction::None => {
                background.0 = Color::srgba(0.22, 0.18, 0.15, 0.85);
            }
        }
    }
}

pub fn toggle_pause_menu_on_escape(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<GameSession>,
    mut resume_events: EventWriter<ResumeGameplayEvent>,
) {
    if !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }

    match session.phase() {
        GamePhase::Playing => {
            let entity = session
                .pause_menu_root
                .unwrap_or_else(|| spawn_pause_menu(&mut commands, &asset_server));
            session.pause_menu_root = Some(entity);
            session.set_phase(GamePhase::Paused);
        }
        GamePhase::Paused => {
            close_pause_menu(&mut commands, session.as_mut());
            resume_events.write(ResumeGameplayEvent);
        }
        GamePhase::MainMenu => {}
    }
}

pub fn handle_pause_menu_interactions(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut interactions: Query<
        (&Interaction, &PauseMenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut resume_events: EventWriter<ResumeGameplayEvent>,
    mut save_events: EventWriter<RequestSaveGameEvent>,
    mut load_events: EventWriter<RequestLoadGameEvent>,
) {
    for (interaction, button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => match button.action {
                PauseMenuAction::Resume => {
                    close_pause_menu(&mut commands, session.as_mut());
                    resume_events.write(ResumeGameplayEvent);
                }
                PauseMenuAction::Save => {
                    save_events.write(RequestSaveGameEvent);
                }
                PauseMenuAction::Load => {
                    load_events.write(RequestLoadGameEvent {
                        from_main_menu: false,
                    });
                }
            },
            Interaction::Hovered => {
                background.0 = Color::srgba(0.35, 0.28, 0.25, 0.9);
            }
            Interaction::None => {
                background.0 = Color::srgba(0.22, 0.18, 0.15, 0.85);
            }
        }
    }
}

pub fn process_save_game_requests(
    mut events: EventReader<RequestSaveGameEvent>,
    session: Res<GameSession>,
    level_state: Option<Res<LevelState>>,
    player_query: Query<
        (
            &Health,
            &PlayerProgression,
            Option<&EquippedWeapon>,
            Option<&EquippedShield>,
        ),
        With<Player>,
    >,
) {
    let mut requested = false;
    for _ in events.read() {
        requested = true;
    }

    if !requested {
        return;
    }

    if matches!(session.phase(), GamePhase::MainMenu) {
        warn!("Cannot save while in the main menu");
        return;
    }

    let Some(level_state) = level_state else {
        warn!("Missing level state resource; cannot save progress");
        return;
    };

    let Some((health, progression, weapon, shield)) = player_query.iter().next() else {
        warn!("Player data not found; cannot save progress");
        return;
    };

    let mut data = GameSaveData::new();
    data.level_index = level_state.current_index();
    data.player_health = health.current;
    data.player_max_health = health.max;
    data.player_level = progression.level;
    data.player_experience = progression.experience;
    data.equipped_weapon = weapon.map(|w| w.kind);
    data.equipped_shield = shield.map(|s| s.kind);

    if let Err(error) = fs::create_dir_all(GameSession::SAVE_DIRECTORY) {
        error!("Failed to create save directory: {error}");
        return;
    }

    match serde_json::to_string_pretty(&data) {
        Ok(serialized) => {
            if let Err(error) = fs::write(GameSession::SAVE_SLOT_FILE, serialized) {
                error!("Failed to write save file: {error}");
            } else {
                info!("Game progress saved to {}", GameSession::SAVE_SLOT_FILE);
            }
        }
        Err(error) => {
            error!("Failed to serialize save data: {error}");
        }
    }
}

pub fn process_load_game_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<GameSession>,
    mut events: EventReader<RequestLoadGameEvent>,
    mut level_state: ResMut<LevelState>,
    mut build_context: ResMut<LevelBuildContext>,
    death_state: Option<ResMut<PlayerDeathState>>,
    mut player_query: Query<
        (
            Entity,
            &mut Health,
            &mut Attack,
            &mut Defense,
            &mut PlayerProgression,
            &mut Sprite,
            Option<&EquippedShield>,
        ),
        With<Player>,
    >,
    mut weapon_events: EventWriter<WeaponEquipEvent>,
    mut shield_events: EventWriter<ShieldEquipEvent>,
) {
    let mut requested = false;
    let mut from_main_menu = false;
    for event in events.read() {
        requested = true;
        if event.from_main_menu {
            from_main_menu = true;
        }
    }

    if !requested {
        return;
    }

    let save_path = Path::new(GameSession::SAVE_SLOT_FILE);
    if !save_path.exists() {
        warn!("No existing save found; cannot load progress");
        if from_main_menu {
            if session.main_menu_root.is_none() {
                let entity = build_main_menu(&mut commands, &asset_server);
                session.main_menu_root = Some(entity);
            }
            session.set_phase(GamePhase::MainMenu);
        }
        return;
    }

    let raw = match fs::read_to_string(save_path) {
        Ok(contents) => contents,
        Err(error) => {
            error!("Failed to read save file: {error}");
            if from_main_menu {
                if session.main_menu_root.is_none() {
                    let entity = build_main_menu(&mut commands, &asset_server);
                    session.main_menu_root = Some(entity);
                }
                session.set_phase(GamePhase::MainMenu);
            }
            return;
        }
    };

    let data: GameSaveData = match serde_json::from_str(&raw) {
        Ok(data) => data,
        Err(error) => {
            error!("Failed to parse save data: {error}");
            if from_main_menu {
                if session.main_menu_root.is_none() {
                    let entity = build_main_menu(&mut commands, &asset_server);
                    session.main_menu_root = Some(entity);
                }
                session.set_phase(GamePhase::MainMenu);
            }
            return;
        }
    };

    if data.version != GameSaveData::CURRENT_VERSION {
        warn!(
            "Save version ({}) differs from current ({}) — attempting to load anyway",
            data.version,
            GameSaveData::CURRENT_VERSION
        );
    }

    let Ok((entity, mut health, mut attack, mut defense, mut progression, mut sprite, shield)) =
        player_query.single_mut()
    else {
        warn!("Player entity not found; cannot apply save data");
        return;
    };

    let level_count = level_state.definition_count().max(1);
    let target_index = data.level_index.min(level_count.saturating_sub(1));
    level_state.set_current_index(target_index);
    build_context.pending_layout = Some(target_index);
    build_context.pending_finalize = None;

    let clamped_level = data.player_level.min(PlayerProgression::max_level());
    progression.level = clamped_level;

    let xp_cap = progression.next_level_requirement().unwrap_or(0);
    progression.experience = if xp_cap == 0 {
        0
    } else {
        data.player_experience.min(xp_cap)
    };

    attack.base = progression.base_attack();
    defense.base = progression.base_defense();

    sprite.image = asset_server.load(progression.sprite_path());

    health.max = data.player_max_health.max(1);
    health.current = data.player_health.clamp(0, health.max);

    if let Some(kind) = data.equipped_weapon {
        weapon_events.write(WeaponEquipEvent { kind });
    }

    match data.equipped_shield {
        Some(kind) => {
            shield_events.write(ShieldEquipEvent { kind });
        }
        None => {
            if let Some(shield_component) = shield {
                defense.adjust_bonus(-shield_component.defense_bonus);
                commands.entity(entity).remove::<EquippedShield>();
            }
        }
    }

    commands.entity(entity).remove::<PlayerDead>();
    commands.entity(entity).remove::<Poisoned>();

    if let Some(mut death_state) = death_state {
        if let Some(screen_entity) = death_state.screen_entity.take() {
            commands.entity(screen_entity).despawn();
        }
        death_state.clear_screen();
        death_state.clear_timer();
    }

    close_main_menu(&mut commands, session.as_mut());
    close_pause_menu(&mut commands, session.as_mut());
    session.set_phase(GamePhase::Playing);

    info!(
        "Loaded save: level {}, HP {}/{}, Lv {}, weapon {:?}, shield {:?}",
        target_index + 1,
        health.current,
        health.max,
        progression.level,
        data.equipped_weapon,
        data.equipped_shield
    );
}

pub fn resume_gameplay(
    mut session: ResMut<GameSession>,
    mut resume_events: EventReader<ResumeGameplayEvent>,
) {
    let mut requested = false;
    for _ in resume_events.read() {
        requested = true;
    }

    if requested {
        session.set_phase(GamePhase::Playing);
    }
}

pub fn activate_gameplay_after_start(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    mut events: EventReader<StartNewGameEvent>,
) {
    let mut triggered = false;
    for _ in events.read() {
        triggered = true;
    }

    if triggered {
        session.set_phase(GamePhase::Playing);
        close_main_menu(&mut commands, session.as_mut());
    }
}

fn build_main_menu(commands: &mut Commands, asset_server: &AssetServer) -> Entity {
    let font = asset_server.load(MENU_FONT_PATH);

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(
                MENU_OVERLAY_COLOR[0],
                MENU_OVERLAY_COLOR[1],
                MENU_OVERLAY_COLOR[2],
                MENU_OVERLAY_COLOR[3],
            )),
            GlobalZIndex(100),
            Name::new("MainMenuRoot"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(16.0),
                        ..Default::default()
                    },
                    Name::new("MainMenuPanel"),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Dungeon of Rust"),
                        TextFont {
                            font: font.clone(),
                            font_size: MENU_TITLE_FONT_SIZE,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.95, 0.93, 0.9)),
                        Name::new("MainMenuTitle"),
                    ));

                    panel
                        .spawn((
                            Button,
                            MainMenuButton {
                                action: MainMenuAction::NewGame,
                            },
                            Node {
                                width: Val::Px(MENU_BUTTON_WIDTH),
                                height: Val::Px(MENU_BUTTON_HEIGHT),
                                padding: UiRect::all(Val::Px(12.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..Default::default()
                            },
                            BorderColor(Color::srgba(0.65, 0.6, 0.5, 0.6)),
                            BackgroundColor(Color::srgba(0.22, 0.18, 0.15, 0.85)),
                            Name::new("MainMenuButtonNewGame"),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("New Game"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: MENU_BUTTON_FONT_SIZE,
                                    ..Default::default()
                                },
                                TextColor(Color::srgb(0.95, 0.93, 0.9)),
                            ));
                        });

                    panel
                        .spawn((
                            Button,
                            MainMenuButton {
                                action: MainMenuAction::LoadGame,
                            },
                            Node {
                                width: Val::Px(MENU_BUTTON_WIDTH),
                                height: Val::Px(MENU_BUTTON_HEIGHT),
                                padding: UiRect::all(Val::Px(12.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..Default::default()
                            },
                            BorderColor(Color::srgba(0.65, 0.6, 0.5, 0.6)),
                            BackgroundColor(Color::srgba(0.22, 0.18, 0.15, 0.85)),
                            Name::new("MainMenuButtonLoadGame"),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Load Game"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: MENU_BUTTON_FONT_SIZE,
                                    ..Default::default()
                                },
                                TextColor(Color::srgb(0.95, 0.93, 0.9)),
                            ));
                        });
                });
        })
        .id()
}

fn spawn_pause_menu(commands: &mut Commands, asset_server: &AssetServer) -> Entity {
    let font = asset_server.load(MENU_FONT_PATH);

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(
                MENU_OVERLAY_COLOR[0],
                MENU_OVERLAY_COLOR[1],
                MENU_OVERLAY_COLOR[2],
                MENU_OVERLAY_COLOR[3],
            )),
            GlobalZIndex(110),
            Name::new("PauseMenuRoot"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(380.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(14.0),
                        ..Default::default()
                    },
                    Name::new("PauseMenuPanel"),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Game Paused"),
                        TextFont {
                            font: font.clone(),
                            font_size: MENU_TITLE_FONT_SIZE * 0.8,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(0.95, 0.93, 0.9)),
                        Name::new("PauseMenuTitle"),
                    ));

                    let actions = [
                        (PauseMenuAction::Resume, "Resume"),
                        (PauseMenuAction::Save, "Save Game"),
                        (PauseMenuAction::Load, "Load Game"),
                    ];

                    for (action, label) in actions {
                        panel
                            .spawn((
                                Button,
                                PauseMenuButton { action },
                                Node {
                                    width: Val::Px(MENU_BUTTON_WIDTH),
                                    height: Val::Px(MENU_BUTTON_HEIGHT),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..Default::default()
                                },
                                BorderColor(Color::srgba(0.65, 0.6, 0.5, 0.6)),
                                BackgroundColor(Color::srgba(0.22, 0.18, 0.15, 0.85)),
                                Name::new(format!("PauseMenuButton_{:?}", action)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: MENU_BUTTON_FONT_SIZE,
                                        ..Default::default()
                                    },
                                    TextColor(Color::srgb(0.95, 0.93, 0.9)),
                                ));
                            });
                    }
                });
        })
        .id()
}

fn close_pause_menu(commands: &mut Commands, session: &mut GameSession) {
    if let Some(entity) = session.pause_menu_root.take() {
        commands.entity(entity).despawn();
    }
}

fn close_main_menu(commands: &mut Commands, session: &mut GameSession) {
    if let Some(entity) = session.main_menu_root.take() {
        commands.entity(entity).despawn();
    }
}
