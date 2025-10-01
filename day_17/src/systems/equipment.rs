use crate::components::*;
use crate::constants::*;
use bevy::prelude::*;

#[derive(Event, Debug, Clone, Copy)]
pub struct ShieldEquipEvent {
    pub kind: ShieldKind,
}

pub fn handle_shield_equip_events(
    mut commands: Commands,
    mut events: EventReader<ShieldEquipEvent>,
    mut player_query: Query<
        (
            Entity,
            &mut Defense,
            Option<&EquippedShield>,
            Option<&Children>,
        ),
        With<Player>,
    >,
    asset_server: Res<AssetServer>,
    shield_visuals: Query<Entity, With<ShieldVisual>>,
) {
    let Some((player_entity, mut defense, equipped, children)) = player_query.iter_mut().next()
    else {
        events.clear();
        return;
    };

    let mut current_bonus = equipped.map(|shield| shield.defense_bonus).unwrap_or(0);
    let mut cleaned_visuals = false;
    let mut equipped_any = false;

    for event in events.read() {
        let new_bonus = event.kind.defense_bonus();
        let delta = new_bonus - current_bonus;

        if delta != 0 {
            defense.adjust_bonus(delta);
        }

        commands
            .entity(player_entity)
            .insert(EquippedShield::new(event.kind));

        if !cleaned_visuals {
            if let Some(children) = children {
                for child in children.iter() {
                    if shield_visuals.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
            cleaned_visuals = true;
        }

        let shield_entity = commands
            .spawn((
                ShieldVisual,
                Sprite::from_image(asset_server.load(event.kind.sprite_path())),
                Transform::from_translation(Vec3::new(SHIELD_OFFSET_X, SHIELD_OFFSET_Y, SHIELD_Z))
                    .with_scale(Vec3::splat(SHIELD_SCALE)),
                Name::new(format!("Equipped{}", event.kind.display_name())),
            ))
            .id();

        commands.entity(player_entity).add_child(shield_entity);

        current_bonus = new_bonus;
        equipped_any = true;
    }

    if equipped_any {
        info!("盾牌裝備完成，當前防禦：{}", defense.value());
    }
}
