use crate::systems::*;
use bevy::prelude::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerPickupEvent>()
            .add_systems(PostStartup, spawn_random_pickups)
            .add_systems(Update, player_pickup_detection_system);
    }
}
