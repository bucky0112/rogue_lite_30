use bevy::prelude::*;

mod constants;
mod components;
mod systems;
mod plugins;

use plugins::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PlayerPlugin)
        .run();
}
