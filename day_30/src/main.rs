use bevy::prelude::*;

macro_rules! dev_info {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            bevy::log::info!($($arg)*);
        }
    };
}

mod components;
mod constants;
mod plugins;
mod resources;
mod systems;

use plugins::*;

fn main() {
    let mut app = App::new();

    // 針對 WASM 目標設定 WindowPlugin
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
    );

    // 桌面平台使用預設設定
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));

    app.add_plugins(SessionPlugin)
        .add_plugins((
            WorldPlugin,
            LevelPlugin,
            AudioPlugin,
            PlayerPlugin,
            UiPlugin,
            EnemyPlugin,
            ProgressionPlugin,
            ItemPlugin,
            ChestPlugin,
            EquipmentPlugin,
            CameraPlugin,
            AttackPlugin,
            WallCollisionPlugin,
            DoorInteractionPlugin,
            RoomTransitionPlugin,
        ))
        .add_plugins(EffectsPlugin)
        .run();
}
