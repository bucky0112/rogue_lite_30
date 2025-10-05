use bevy::prelude::*;

use crate::components::items::PickupEffect;

#[derive(Component)]
pub struct Chest {
    pub contents: ChestContents,
    pub state: ChestState,
}

impl Chest {
    pub fn new(contents: ChestContents) -> Self {
        Self {
            contents,
            state: ChestState::Closed,
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.state, ChestState::Closed)
    }
}

#[derive(Clone)]
pub enum ChestContents {
    Item(PickupEffect),
    Mimic,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChestState {
    Closed,
    RevealingItem,
    Empty,
    MimicAwakened,
}

#[derive(Component)]
pub struct ChestItemReveal {
    pub timer: Timer,
    pub effect: PickupEffect,
}

impl ChestItemReveal {
    pub fn new(duration: f32, effect: PickupEffect) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            effect,
        }
    }
}

#[derive(Component)]
pub struct ChestItemVisual;
