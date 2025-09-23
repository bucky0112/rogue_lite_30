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
            PlayerPlugin,
            EnemyPlugin,
            CameraPlugin,
            AttackPlugin,
            WallCollisionPlugin,
            DoorInteractionPlugin,
            RoomTransitionPlugin,
        ))
        .run();
}
