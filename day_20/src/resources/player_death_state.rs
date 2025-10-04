use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerDeathState {
    pub timer: Option<Timer>,
    pub screen_entity: Option<Entity>,
}

impl PlayerDeathState {
    pub fn is_active(&self) -> bool {
        self.timer.is_some()
    }

    pub fn start(&mut self, duration: f32) {
        if self.timer.is_some() {
            return;
        }

        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.reset();
        self.timer = Some(timer);
    }

    pub fn clear_timer(&mut self) {
        self.timer = None;
    }

    pub fn clear_screen(&mut self) {
        self.screen_entity = None;
    }
}
