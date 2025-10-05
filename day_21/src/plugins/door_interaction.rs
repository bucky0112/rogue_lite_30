use crate::systems::{
    DoorInteractionEvent, DoorStateChangedEvent, door_interaction_system, input_system,
};
use bevy::prelude::*;

pub struct DoorInteractionPlugin;

impl Plugin for DoorInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DoorInteractionEvent>()
            .add_event::<DoorStateChangedEvent>()
            .add_systems(Update, (input_system, door_interaction_system));
    }
}
