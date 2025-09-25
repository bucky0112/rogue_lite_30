use crate::systems::{TransitionCooldown, room_transition_system};
use bevy::prelude::*;

pub struct RoomTransitionPlugin;

impl Plugin for RoomTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitionCooldown>()
            .add_systems(Update, room_transition_system);
    }
}
