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

/// 玩家／敵人共享的耐力值，用來限制爆發行為
#[derive(Component, Debug, Clone)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen_per_second: f32,
}

impl Stamina {
    pub fn new(max: f32, regen_per_second: f32) -> Self {
        Self {
            current: max,
            max,
            regen_per_second,
        }
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= f32::EPSILON {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn spend(&mut self, amount: f32) -> bool {
        if self.current < amount {
            return false;
        }

        self.current -= amount;
        true
    }

    pub fn regen(&mut self, delta_seconds: f32) {
        if self.current >= self.max {
            return;
        }

        self.current = (self.current + self.regen_per_second * delta_seconds).min(self.max);
    }

    pub fn refill(&mut self) {
        self.current = self.max;
    }
}

/// 中毒狀態，會定期造成 HP 損失
#[derive(Component, Debug)]
pub struct Poisoned {
    pub tick_timer: Timer,
    pub damage_per_tick: i32,
}

impl Poisoned {
    pub fn new(tick_seconds: f32, damage_per_tick: i32) -> Self {
        Self {
            tick_timer: Timer::from_seconds(tick_seconds, TimerMode::Repeating),
            damage_per_tick,
        }
    }

    pub fn reset_timer(&mut self) {
        self.tick_timer.reset();
    }
}
