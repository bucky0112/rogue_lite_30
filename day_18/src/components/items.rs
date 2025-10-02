use bevy::prelude::*;

use crate::components::equipment::{ShieldKind, WeaponKind};

#[derive(Component, Debug, Clone)]
pub struct Pickup {
    pub effect: PickupEffect,
}

#[derive(Debug, Clone)]
pub enum PickupEffect {
    Heal(i32),
    RestoreStamina(f32),
    CurePoison,
    EquipShield(ShieldKind),
    EquipWeapon(WeaponKind),
}

impl Pickup {
    pub fn new(effect: PickupEffect) -> Self {
        Self { effect }
    }
}
