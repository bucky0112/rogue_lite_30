use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn spawn_random_pickups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles: Query<(&Transform, &RoomTile), Without<Pickup>>, // Room tiles already spawn before PostStartup
) {
    let mut floor_positions: Vec<Vec3> = tiles
        .iter()
        .filter(|(_transform, tile)| {
            matches!(
                tile.tile_type,
                RoomTileType::Floor | RoomTileType::FloorOutdoor
            )
        })
        .map(|(transform, _)| transform.translation)
        .collect();

    if floor_positions.is_empty() {
        return;
    }

    let mut rng = thread_rng();
    floor_positions.shuffle(&mut rng);

    let pickups_to_spawn = ITEM_RANDOM_PICKUP_COUNT.min(floor_positions.len());
    if pickups_to_spawn == 0 {
        return;
    }

    let effects_cycle = [
        PickupEffect::Heal(ITEM_HEALTH_POTION_HEAL_AMOUNT),
        PickupEffect::RestoreStamina(ITEM_STAMINA_POTION_AMOUNT),
        PickupEffect::CurePoison,
    ];

    for (index, position) in floor_positions
        .into_iter()
        .take(pickups_to_spawn)
        .enumerate()
    {
        let effect = effects_cycle[index % effects_cycle.len()].clone();
        let (sprite_path, name_label) = match effect {
            PickupEffect::Heal(_) => ("items/potions/health.png", "PickupHealth"),
            PickupEffect::RestoreStamina(_) => ("items/potions/stamina.png", "PickupStamina"),
            PickupEffect::CurePoison => ("items/potions/toxic.png", "PickupAntidote"),
        };

        commands.spawn((
            Pickup::new(effect),
            Sprite::from_image(asset_server.load(sprite_path)),
            Transform::from_translation(Vec3::new(
                position.x,
                position.y + ITEM_PICKUP_Z_OFFSET,
                ITEM_PICKUP_Z,
            ))
            .with_scale(Vec3::splat(ITEM_PICKUP_SCALE)),
            Name::new(name_label),
        ));
    }
}

pub fn player_pickup_detection_system(
    mut commands: Commands,
    mut player_query: Query<
        (
            &Transform,
            &mut Health,
            Option<Mut<Stamina>>,
            Option<&Poisoned>,
            Entity,
        ),
        (With<Player>, Without<PlayerDead>),
    >,
    pickup_query: Query<(Entity, &Transform, &Pickup)>,
) {
    let Some((player_transform, mut health, mut stamina, poison_state, player_entity)) =
        player_query.iter_mut().next()
    else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (pickup_entity, pickup_transform, pickup) in &pickup_query {
        let item_pos = pickup_transform.translation.truncate();
        if player_pos.distance(item_pos) > ITEM_PICKUP_DISTANCE {
            continue;
        }

        match &pickup.effect {
            PickupEffect::Heal(amount) => {
                let before = health.current;
                health.current = (health.current + amount).min(health.max);
                info!("拾取紅色藥水：HP {} -> {}", before, health.current);
            }
            PickupEffect::RestoreStamina(amount) => {
                if let Some(stamina_ref) = stamina.as_mut() {
                    let before = stamina_ref.current;
                    stamina_ref.current = (stamina_ref.current + amount).min(stamina_ref.max);
                    info!(
                        "拾取綠色藥水：耐力 {:.1} -> {:.1}",
                        before, stamina_ref.current
                    );
                } else {
                    info!("拾取綠色藥水：目前沒有耐力屬性可恢復");
                }
            }
            PickupEffect::CurePoison => {
                if poison_state.is_some() {
                    commands.entity(player_entity).remove::<Poisoned>();
                    info!("拾取解毒藥水：中毒狀態已解除");
                } else {
                    info!("拾取解毒藥水：目前沒有中毒狀態");
                }
            }
        }

        commands.entity(pickup_entity).despawn();
    }
}
