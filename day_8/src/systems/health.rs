use crate::components::*;
use bevy::prelude::*;

pub fn health_system(query: Query<&Health, With<Player>>) {
    for health in &query {
        if health.current <= 0 {
            info!("玩家死亡！");
        }
    }
}
