use crate::components::*;
use crate::constants::*;
use crate::systems::equipment::{ShieldEquipEvent, WeaponEquipEvent};
use crate::systems::items::{PlayerPickupEvent, pickup_visual_for_effect};
use bevy::prelude::*;

#[derive(Event, Debug, Clone, Copy)]
pub struct ChestInteractionEvent {
    pub chest: Entity,
}

pub fn spawn_weapon_demo_chests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles: Query<(&Transform, &RoomTile), Without<Chest>>,
) {
    let mut positions: Vec<Vec3> = tiles
        .iter()
        .filter_map(|(transform, tile)| match tile.tile_type {
            RoomTileType::Floor | RoomTileType::FloorOutdoor => Some(transform.translation),
            _ => None,
        })
        .collect();

    if positions.is_empty() {
        return;
    }

    positions.sort_by(|a, b| {
        let origin = Vec2::ZERO;
        let da = a.truncate().distance(origin);
        let db = b.truncate().distance(origin);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });

    let chest_loadout = [
        (
            ChestContents::Item(PickupEffect::EquipShield(ShieldKind::Level1)),
            ShieldKind::Level1.display_name(),
        ),
        (
            ChestContents::Item(PickupEffect::EquipShield(ShieldKind::Level2)),
            ShieldKind::Level2.display_name(),
        ),
        (
            ChestContents::Item(PickupEffect::EquipWeapon(WeaponKind::Level2)),
            WeaponKind::Level2.display_name(),
        ),
        (
            ChestContents::Item(PickupEffect::EquipWeapon(WeaponKind::Level3)),
            WeaponKind::Level3.display_name(),
        ),
        (
            ChestContents::Item(PickupEffect::EquipWeapon(WeaponKind::Level4)),
            WeaponKind::Level4.display_name(),
        ),
        (
            ChestContents::Item(PickupEffect::EquipWeapon(WeaponKind::Level5)),
            WeaponKind::Level5.display_name(),
        ),
    ];

    for (index, (contents, label)) in chest_loadout.iter().enumerate() {
        let Some(base_position) = positions.get(index) else {
            break;
        };
        commands.spawn((
            Chest::new(contents.clone()),
            Sprite::from_image(asset_server.load("items/chests/chest_closed.png")),
            Transform::from_translation(Vec3::new(base_position.x, base_position.y, CHEST_Z))
                .with_scale(Vec3::splat(CHEST_SCALE)),
            Name::new(format!("{}Chest", label)),
        ));
    }
}

pub fn chest_interaction_system(
    mut commands: Commands,
    mut events: EventReader<ChestInteractionEvent>,
    mut chests: Query<(Entity, &Transform, &mut Chest, &mut Sprite)>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let Ok((entity, transform, mut chest, mut sprite)) = chests.get_mut(event.chest) else {
            continue;
        };

        if !chest.is_closed() {
            continue;
        }

        match chest.contents.clone() {
            ChestContents::Item(effect) => {
                chest.state = ChestState::RevealingItem;
                sprite.image = asset_server.load("items/chests/chest_open.png");

                commands.entity(entity).insert(ChestItemReveal::new(
                    CHEST_ITEM_REVEAL_SECONDS,
                    effect.clone(),
                ));

                let (sprite_path, _) = pickup_visual_for_effect(&effect);
                let item_visual = commands
                    .spawn((
                        ChestItemVisual,
                        Sprite::from_image(asset_server.load(sprite_path)),
                        Transform::from_translation(Vec3::new(0.0, CHEST_ITEM_DISPLAY_OFFSET, 0.1))
                            .with_scale(Vec3::splat(CHEST_ITEM_DISPLAY_SCALE)),
                        Name::new("ChestItemVisual"),
                    ))
                    .id();

                commands.entity(entity).add_child(item_visual);
                info!("寶箱開啟，顯示道具：{:?}", effect);
            }
            ChestContents::Mimic => {
                chest.state = ChestState::MimicAwakened;
                sprite.image = asset_server.load("items/chests/chest_mimic.png");

                let origin = transform.translation;
                commands.entity(entity).insert((
                    Enemy,
                    Mimic,
                    Health::new(CHEST_MIMIC_HEALTH),
                    Attack::new(CHEST_MIMIC_ATTACK),
                    Defense::new(CHEST_MIMIC_DEFENSE),
                    EnemyAIState {
                        state: EnemyBehaviorState::Chasing,
                    },
                    EnemyPatrol {
                        origin,
                        range: CHEST_MIMIC_PATROL_RANGE,
                        direction: 1.0,
                    },
                    EnemyAlert {
                        trigger_radius: CHEST_MIMIC_TRIGGER_RADIUS,
                        leash_radius: CHEST_MIMIC_LEASH_RADIUS,
                    },
                    EnemySpeeds {
                        patrol: 0.0,
                        chase: CHEST_MIMIC_CHASE_SPEED,
                    },
                    EnemyAttack {
                        radius: CHEST_MIMIC_ATTACK_RADIUS,
                        cooldown: {
                            let mut timer = Timer::from_seconds(
                                CHEST_MIMIC_ATTACK_COOLDOWN,
                                TimerMode::Repeating,
                            );
                            timer.set_elapsed(timer.duration());
                            timer
                        },
                    },
                    Name::new("ChestMimic"),
                ));
                info!("寶箱竟是假扮的 mimic！開始追擊玩家");
            }
        }
    }
}

pub fn chest_item_reveal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut reveal_query: Query<(
        Entity,
        &mut ChestItemReveal,
        &mut Chest,
        &mut Sprite,
        Option<&Children>,
    )>,
    mut player_query: Query<
        (Entity, &mut Health, Option<Mut<Stamina>>, Option<&Poisoned>),
        (With<Player>, Without<PlayerDead>),
    >,
    item_visuals: Query<Entity, With<ChestItemVisual>>,
    asset_server: Res<AssetServer>,
    mut shield_events: EventWriter<ShieldEquipEvent>,
    mut weapon_events: EventWriter<WeaponEquipEvent>,
    mut pickup_events: EventWriter<PlayerPickupEvent>,
) {
    let delta = time.delta();

    for (entity, mut reveal, mut chest, mut sprite, children) in &mut reveal_query {
        if !matches!(chest.state, ChestState::RevealingItem) {
            continue;
        }

        reveal.timer.tick(delta);
        if !reveal.timer.finished() {
            continue;
        }

        let Some((player_entity, mut health, stamina, poison_state)) =
            player_query.iter_mut().next()
        else {
            continue;
        };

        apply_pickup_effect(
            &mut commands,
            player_entity,
            &mut health,
            stamina,
            poison_state,
            &reveal.effect,
            &mut shield_events,
            &mut weapon_events,
            &mut pickup_events,
        );

        chest.state = ChestState::Empty;
        sprite.image = asset_server.load("items/chests/chest_empty.png");

        if let Some(children) = children {
            for child in children.iter() {
                if item_visuals.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        commands.entity(entity).remove::<ChestItemReveal>();
        info!("寶箱中的道具已被玩家使用");
    }
}

fn apply_pickup_effect(
    commands: &mut Commands,
    player_entity: Entity,
    health: &mut Health,
    stamina: Option<Mut<Stamina>>,
    poison_state: Option<&Poisoned>,
    effect: &PickupEffect,
    shield_events: &mut EventWriter<ShieldEquipEvent>,
    weapon_events: &mut EventWriter<WeaponEquipEvent>,
    pickup_events: &mut EventWriter<PlayerPickupEvent>,
) {
    pickup_events.write(PlayerPickupEvent);
    match effect {
        PickupEffect::Heal(amount) => {
            let before = health.current;
            health.current = (health.current + amount).min(health.max);
            info!("寶箱道具：恢復生命 {} -> {}", before, health.current);
        }
        PickupEffect::RestoreStamina(amount) => {
            if let Some(mut stamina_ref) = stamina {
                let before = stamina_ref.current;
                stamina_ref.current = (stamina_ref.current + amount).min(stamina_ref.max);
                info!(
                    "寶箱道具：恢復耐力 {:.1} -> {:.1}",
                    before, stamina_ref.current
                );
            } else {
                info!("寶箱道具：目前沒有耐力屬性可恢復");
            }
        }
        PickupEffect::CurePoison => {
            if poison_state.is_some() {
                commands.entity(player_entity).remove::<Poisoned>();
                info!("寶箱道具：中毒狀態已解除");
            } else {
                info!("寶箱道具：玩家目前沒有中毒狀態");
            }
        }
        PickupEffect::EquipShield(kind) => {
            shield_events.write(ShieldEquipEvent { kind: *kind });
            info!("寶箱道具：裝備 {:?} 提升防禦", kind);
        }
        PickupEffect::EquipWeapon(kind) => {
            weapon_events.write(WeaponEquipEvent { kind: *kind });
            info!("寶箱道具：裝備 {:?} 提升攻擊", kind);
        }
    }
}
