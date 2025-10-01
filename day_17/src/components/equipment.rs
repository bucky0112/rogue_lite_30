use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ShieldKind {
    Level1,
    Level2,
}

impl ShieldKind {
    pub fn defense_bonus(&self) -> i32 {
        match self {
            ShieldKind::Level1 => 4,
            ShieldKind::Level2 => 8,
        }
    }

    pub fn sprite_path(&self) -> &'static str {
        match self {
            ShieldKind::Level1 => "armors/shield_lv1.png",
            ShieldKind::Level2 => "armors/shield_lv2.png",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ShieldKind::Level1 => "ShieldLv1",
            ShieldKind::Level2 => "ShieldLv2",
        }
    }
}

#[derive(Component, Debug)]
pub struct EquippedShield {
    pub kind: ShieldKind,
    pub defense_bonus: i32,
}

impl EquippedShield {
    pub fn new(kind: ShieldKind) -> Self {
        Self {
            defense_bonus: kind.defense_bonus(),
            kind,
        }
    }
}

#[derive(Component)]
pub struct ShieldVisual;
