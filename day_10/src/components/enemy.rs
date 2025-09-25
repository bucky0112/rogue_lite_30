use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Slime;

#[derive(Component)]
pub struct Cyclops;

#[derive(Component)]
pub struct EnemyAIState {
    pub state: EnemyBehaviorState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyBehaviorState {
    Patrolling,
    Chasing,
    WindUp,
    Charging,
}

#[derive(Component)]
pub struct EnemyPatrol {
    pub origin: Vec3,
    pub range: f32,
    pub direction: f32,
}

impl EnemyPatrol {
    pub fn bounds(&self) -> (f32, f32) {
        (self.origin.x - self.range, self.origin.x + self.range)
    }
}

#[derive(Component)]
pub struct EnemyAlert {
    pub trigger_radius: f32,
    pub leash_radius: f32,
}

#[derive(Component)]
pub struct EnemySpeeds {
    pub patrol: f32,
    pub chase: f32,
}

#[derive(Component)]
pub struct EnemyAttack {
    pub damage: i32,
    pub radius: f32,
    pub cooldown: Timer,
}

#[derive(Component)]
pub struct CyclopsCharge {
    pub windup: Timer,
    pub charge: Timer,
    pub cooldown: Timer,
    pub facing: Vec2,
    pub ready: bool,
}
