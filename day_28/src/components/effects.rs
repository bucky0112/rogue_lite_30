use bevy::prelude::*;

#[derive(Component)]
pub struct HitSpark {
    pub timer: Timer,
    pub max_scale: f32,
}

impl HitSpark {
    pub fn new(duration: f32, max_scale: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            max_scale,
        }
    }
}

#[derive(Component)]
pub struct EnemyHitFlash {
    pub timer: Timer,
    pub blink_timer: Timer,
    pub highlighted: bool,
}

impl EnemyHitFlash {
    pub fn new(duration: f32, blink_interval: f32) -> Self {
        let mut blink_timer = Timer::from_seconds(blink_interval, TimerMode::Repeating);
        blink_timer.set_elapsed(std::time::Duration::from_secs_f32(0.0));
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            blink_timer,
            highlighted: true,
        }
    }
}

#[derive(Component)]
pub struct DeathParticle {
    pub velocity: Vec2,
    pub timer: Timer,
    pub start_scale: Vec3,
}

impl DeathParticle {
    pub fn new(velocity: Vec2, lifetime: f32, scale: Vec3) -> Self {
        Self {
            velocity,
            timer: Timer::from_seconds(lifetime, TimerMode::Once),
            start_scale: scale,
        }
    }
}
