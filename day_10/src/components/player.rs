use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Component)]
pub struct PlayerFacing {
    pub direction: Vec2,
}

impl PlayerFacing {
    pub fn new() -> Self {
        Self {
            direction: Vec2::new(1.0, 0.0), // Default facing right
        }
    }
}

#[derive(Component)]
pub struct Weapon;

#[derive(Component)]
pub struct WeaponSprites {
    pub right_sprite: Handle<Image>,
    pub left_sprite: Handle<Image>,
}

#[derive(Component)]
pub struct WeaponOffset {
    pub base_angle: f32,
    pub position: Vec2,
}

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: Timer,
    pub from_angle: f32,
    pub to_angle: f32,
}

#[derive(Component)]
pub struct InputVector(pub Vec2);

#[derive(Component)]
pub struct DamageFlash {
    pub timer: Timer,
    pub flashes_remaining: u8,
    pub show_highlight: bool,
    pub highlight_color: Color,
}

impl DamageFlash {
    pub fn new(flashes: u8, interval: f32, highlight_color: Color) -> Self {
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            flashes_remaining: flashes,
            show_highlight: true,
            highlight_color,
        }
    }

    pub fn restart(&mut self, flashes: u8) {
        self.timer.reset();
        self.flashes_remaining = flashes;
        self.show_highlight = true;
    }
}
