use crate::systems::*;
use bevy::prelude::*;

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChestInteractionEvent>()
            .add_systems(PostStartup, spawn_shield_demo_chests)
            .add_systems(Update, (chest_interaction_system, chest_item_reveal_system));
    }
}
