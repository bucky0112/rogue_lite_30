use bevy::prelude::*;

mod components;
mod constants;
mod plugins;
mod resources;
mod systems;

use plugins::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            WorldPlugin,
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
        .run();
}
