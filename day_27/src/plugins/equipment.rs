use crate::systems::*;
use bevy::prelude::*;

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShieldEquipEvent>().add_systems(
            Update,
            handle_shield_equip_events
                .after(chest_item_reveal_system)
                .after(player_pickup_detection_system),
        );
        app.add_event::<WeaponEquipEvent>().add_systems(
            Update,
            handle_weapon_equip_events
                .after(chest_item_reveal_system)
                .after(player_pickup_detection_system),
        );
    }
}
