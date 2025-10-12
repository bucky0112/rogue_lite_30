use crate::components::{DeathParticle, Enemy, EnemyHitFlash, HitSpark};
use crate::resources::CameraShake;
use crate::systems::EnemyHitEvent;
use bevy::prelude::*;
use rand::prelude::*;

const HIT_SPARK_LIFETIME: f32 = 0.18;
const HIT_SPARK_MAX_SCALE: f32 = 1.8;
const HIT_FLASH_DURATION: f32 = 0.24;
const HIT_FLASH_BLINK_INTERVAL: f32 = 0.04;
const PARTICLE_LIFETIME: f32 = 0.6;
const PARTICLE_COUNT: usize = 9;

pub fn spawn_hit_spark_system(mut commands: Commands, mut events: EventReader<EnemyHitEvent>) {
    for event in events.read() {
        if event.damage <= 0 {
            continue;
        }

        let mut spark_transform = Transform::from_translation(event.position);
        spark_transform.translation.z += 20.0;

        commands.spawn((
            HitSpark::new(HIT_SPARK_LIFETIME, HIT_SPARK_MAX_SCALE),
            Sprite {
                color: Color::srgba(1.0, 0.86, 0.48, 0.95),
                custom_size: Some(Vec2::new(18.0, 18.0)),
                ..Default::default()
            },
            spark_transform,
            Name::new("HitSpark"),
        ));
    }
}

pub fn update_hit_spark_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitSpark, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut spark, mut transform, mut sprite) in &mut query {
        spark.timer.tick(time.delta());

        let duration = spark.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (spark.timer.elapsed_secs() / duration).clamp(0.0, 1.0);

        let scale = 1.0 + (spark.max_scale - 1.0) * progress;
        transform.scale = Vec3::splat(scale * 0.6);

        sprite.color.set_alpha((1.0 - progress).clamp(0.0, 1.0));

        if spark.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_enemy_hit_flash_system(
    mut commands: Commands,
    mut events: EventReader<EnemyHitEvent>,
    mut enemy_sprites: Query<&mut Sprite, With<Enemy>>,
) {
    for event in events.read() {
        if let Ok(mut sprite) = enemy_sprites.get_mut(event.entity) {
            let alpha = sprite.color.alpha();
            sprite.color = Color::srgba(1.0, 0.65, 0.65, alpha);
            commands.entity(event.entity).insert(EnemyHitFlash::new(
                HIT_FLASH_DURATION,
                HIT_FLASH_BLINK_INTERVAL,
            ));
        }
    }
}

pub fn update_enemy_hit_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyHitFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in &mut query {
        flash.timer.tick(time.delta());
        flash.blink_timer.tick(time.delta());

        if flash.blink_timer.just_finished() {
            flash.highlighted = !flash.highlighted;
            let alpha = sprite.color.alpha();
            if flash.highlighted {
                sprite.color = Color::srgba(1.0, 0.6, 0.6, alpha);
            } else {
                sprite.color = Color::srgba(1.0, 0.9, 0.9, alpha);
            }
        }

        if flash.timer.finished() {
            let alpha = sprite.color.alpha();
            sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
            commands.entity(entity).remove::<EnemyHitFlash>();
        }
    }
}

pub fn spawn_enemy_death_particles_system(
    mut commands: Commands,
    mut events: EventReader<EnemyHitEvent>,
) {
    let mut rng = thread_rng();

    for event in events.read() {
        if event.remaining_health > 0 {
            continue;
        }

        for _ in 0..PARTICLE_COUNT {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(140.0..220.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            let scale_factor = rng.gen_range(0.4..0.9);

            let color = Color::srgba(0.95, rng.gen_range(0.4..0.65), rng.gen_range(0.2..0.4), 0.9);

            commands.spawn((
                DeathParticle::new(velocity, PARTICLE_LIFETIME, Vec3::splat(scale_factor)),
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(10.0)),
                    ..Default::default()
                },
                Transform::from_translation(event.position + Vec3::new(0.0, 0.0, 16.0)),
                Name::new("EnemyDeathParticle"),
            ));
        }
    }
}

pub fn update_death_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DeathParticle, &mut Transform, &mut Sprite)>,
) {
    let delta = time.delta_secs();

    for (entity, mut particle, mut transform, mut sprite) in &mut query {
        particle.timer.tick(time.delta());

        transform.translation.x += particle.velocity.x * delta;
        transform.translation.y += particle.velocity.y * delta;

        let duration = particle.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (particle.timer.elapsed_secs() / duration).clamp(0.0, 1.0);
        let damping = 1.0 - progress;

        transform.scale = particle.start_scale * (0.4 + damping * 0.8);
        sprite.color.set_alpha(damping);

        if particle.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn trigger_camera_shake_on_enemy_hit(
    mut events: EventReader<EnemyHitEvent>,
    mut shake: ResMut<CameraShake>,
) {
    for event in events.read() {
        let (amplitude, duration) = if event.remaining_health <= 0 {
            (12.0, 0.35)
        } else {
            (6.0, 0.22)
        };
        shake.trigger(amplitude, duration);
    }
}
