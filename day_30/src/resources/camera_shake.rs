use bevy::math::Vec2;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraShake {
    pub timer: Timer,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    pub offset: Vec2,
    active: bool,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            amplitude: 0.0,
            frequency: 18.0,
            phase: 0.0,
            offset: Vec2::ZERO,
            active: false,
        }
    }
}

impl CameraShake {
    pub fn trigger(&mut self, amplitude: f32, duration: f32) {
        if duration <= 0.0 || amplitude <= 0.0 {
            return;
        }

        self.timer = Timer::from_seconds(duration, TimerMode::Once);
        self.timer.reset();
        self.amplitude = amplitude;
        self.phase = 0.0;
        self.active = true;
    }

    pub fn update(&mut self, delta: f32) {
        if !self.active {
            self.offset = Vec2::ZERO;
            return;
        }

        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        if self.timer.finished() {
            self.active = false;
            self.offset = Vec2::ZERO;
            return;
        }

        let duration = self.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (self.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
        let damping = 1.0 - progress;

        self.phase += delta * self.frequency;

        let phase_radians = self.phase * std::f32::consts::TAU;
        let perpendicular = phase_radians + std::f32::consts::FRAC_PI_2;

        self.offset = Vec2::new(
            phase_radians.sin() * self.amplitude * damping,
            perpendicular.sin() * self.amplitude * damping * 0.6,
        );
    }

    pub fn current_offset(&self) -> Vec2 {
        self.offset
    }
}
