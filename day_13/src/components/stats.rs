use bevy::prelude::*;

/// 攻擊屬性，方便未來做加成或是裝備調整
#[derive(Component, Debug, Clone)]
pub struct Attack {
    pub base: i32,
    pub bonus: i32,
    pub multiplier: f32,
}

impl Attack {
    pub fn new(base: i32) -> Self {
        Self {
            base,
            bonus: 0,
            multiplier: 1.0,
        }
    }

    pub fn value(&self) -> i32 {
        let scaled = (self.base + self.bonus) as f32 * self.multiplier;
        scaled.round() as i32
    }

    pub fn adjust_bonus(&mut self, delta: i32) {
        self.bonus += delta;
    }

    pub fn adjust_multiplier(&mut self, delta: f32) {
        self.multiplier = (self.multiplier + delta).max(0.0);
    }

    pub fn reset_modifiers(&mut self) {
        self.bonus = 0;
        self.multiplier = 1.0;
    }
}

/// 防禦屬性，用來降低受到的傷害
#[derive(Component, Debug, Clone)]
pub struct Defense {
    pub base: i32,
    pub bonus: i32,
    pub multiplier: f32,
}

impl Defense {
    pub fn new(base: i32) -> Self {
        Self {
            base,
            bonus: 0,
            multiplier: 1.0,
        }
    }

    pub fn value(&self) -> i32 {
        let scaled = (self.base + self.bonus) as f32 * self.multiplier;
        scaled.round() as i32
    }

    pub fn adjust_bonus(&mut self, delta: i32) {
        self.bonus += delta;
    }

    pub fn adjust_multiplier(&mut self, delta: f32) {
        self.multiplier = (self.multiplier + delta).max(0.0);
    }

    pub fn reset_modifiers(&mut self) {
        self.bonus = 0;
        self.multiplier = 1.0;
    }
}

pub fn compute_damage(attack: i32, defense: Option<i32>) -> i32 {
    let mitigated = if let Some(defense_value) = defense {
        attack - defense_value.max(0)
    } else {
        attack
    };

    mitigated.max(1)
}
