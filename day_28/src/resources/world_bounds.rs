use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl WorldBounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn clamp_point(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        )
    }

    pub fn clamp_translation(&self, translation: Vec3) -> Vec3 {
        let clamped = self.clamp_point(translation.truncate());
        Vec3::new(clamped.x, clamped.y, translation.z)
    }
}
