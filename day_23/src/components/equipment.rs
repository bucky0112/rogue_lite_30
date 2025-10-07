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

#[derive(Clone, Copy, Debug)]
pub enum WeaponKind {
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl WeaponKind {
    pub fn attack_bonus(&self) -> i32 {
        match self {
            WeaponKind::Level1 => 0,
            WeaponKind::Level2 => 6,
            WeaponKind::Level3 => 12,
            WeaponKind::Level4 => 20,
            WeaponKind::Level5 => 30,
        }
    }

    pub fn right_sprite_path(&self) -> &'static str {
        match self {
            WeaponKind::Level1 => "weapons/lv1.png",
            WeaponKind::Level2 => "weapons/lv2.png",
            WeaponKind::Level3 => "weapons/lv3.png",
            WeaponKind::Level4 => "weapons/lv4.png",
            WeaponKind::Level5 => "weapons/lv5.png",
        }
    }

    pub fn left_sprite_path(&self) -> &'static str {
        // Mirror the right-hand sprite until dedicated left-handed art exists.
        self.right_sprite_path()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WeaponKind::Level1 => "SwordLv1",
            WeaponKind::Level2 => "SwordLv2",
            WeaponKind::Level3 => "SwordLv3",
            WeaponKind::Level4 => "SwordLv4",
            WeaponKind::Level5 => "SwordLv5",
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

#[derive(Component, Debug)]
pub struct EquippedWeapon {
    pub kind: WeaponKind,
    pub attack_bonus: i32,
}

impl EquippedWeapon {
    pub fn new(kind: WeaponKind) -> Self {
        Self {
            attack_bonus: kind.attack_bonus(),
            kind,
        }
    }
}

#[derive(Component)]
pub struct ShieldVisual;
