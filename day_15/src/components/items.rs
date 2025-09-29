use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Pickup {
    pub effect: PickupEffect,
}

#[derive(Debug, Clone)]
pub enum PickupEffect {
    Heal(i32),
    RestoreStamina(f32),
    CurePoison,
}

impl Pickup {
    pub fn new(effect: PickupEffect) -> Self {
        Self { effect }
    }
}
