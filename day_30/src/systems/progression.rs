use crate::components::{Attack, Defense, Player, PlayerProgression};
use bevy::prelude::*;

use super::enemy::EnemyDefeatedEvent;

#[derive(Event, Clone, Copy, Debug)]
pub struct PlayerLevelUpEvent {
    pub new_level: usize,
}

pub fn apply_player_level_up_effects(
    mut level_events: EventReader<PlayerLevelUpEvent>,
    mut player_query: Query<
        (&PlayerProgression, &mut Attack, &mut Defense, &mut Sprite),
        With<Player>,
    >,
    asset_server: Res<AssetServer>,
) {
    let mut latest_level: Option<usize> = None;
    for event in level_events.read() {
        latest_level = Some(event.new_level);
    }

    let Some(level) = latest_level else {
        return;
    };

    let Some((progression, mut attack, mut defense, mut sprite)) = player_query.iter_mut().next()
    else {
        return;
    };

    attack.base = progression.base_attack();
    defense.base = progression.base_defense();

    let sprite_path = progression.sprite_path();
    sprite.image = asset_server.load(sprite_path);

    dev_info!(
        "Player leveled up to Lv.{}! Base ATK {} | Base DEF {}",
        level,
        attack.base,
        defense.base,
    );
}

pub fn apply_enemy_experience_rewards(
    mut defeated_events: EventReader<EnemyDefeatedEvent>,
    mut level_up_events: EventWriter<PlayerLevelUpEvent>,
    mut player_query: Query<&mut PlayerProgression, With<Player>>,
) {
    let Some(mut progression) = player_query.iter_mut().next() else {
        defeated_events.clear();
        return;
    };

    for event in defeated_events.read() {
        if progression.level >= PlayerProgression::max_level() {
            continue;
        }

        let mut remaining_xp = event.experience;

        while remaining_xp > 0 {
            let Some(requirement) = progression.next_level_requirement() else {
                progression.experience = 0;
                break;
            };

            if progression.experience + remaining_xp < requirement {
                progression.experience += remaining_xp;
                break;
            }

            let xp_needed = requirement.saturating_sub(progression.experience);
            remaining_xp = remaining_xp.saturating_sub(xp_needed);
            progression.experience = 0;
            let new_level = (progression.level + 1).min(PlayerProgression::max_level());

            if new_level != progression.level {
                progression.level = new_level;
                level_up_events.write(PlayerLevelUpEvent {
                    new_level: progression.level,
                });
            }

            if progression.level >= PlayerProgression::max_level() {
                progression.experience = 0;
                break;
            }
        }

        if progression.level >= PlayerProgression::max_level() {
            dev_info!(
                "Earned {} EXP from {}, but player already reached Lv.{} cap",
                event.experience,
                event.enemy_name,
                progression.level,
            );
        } else if let Some(requirement) = progression.next_level_requirement() {
            dev_info!(
                "Earned {} EXP from {}; now {}/{} (Lv.{})",
                event.enemy_name,
                event.experience,
                progression.experience,
                requirement,
                progression.level,
            );
        }
    }
}
