use crate::systems::*;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_slime, spawn_cyclops))
            .add_systems(Update, (slime_ai_system, cyclops_ai_system));
    }
}
