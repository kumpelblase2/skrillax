use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, NetworkedEntity};
use bevy_ecs::prelude::*;
use cgmath::prelude::*;
use silkroad_protocol::world::{
    ActionState, ActiveScroll, AliveState, BodyState, EntityDespawn, EntityRarity, EntitySpawn, EntityState,
    EntityTypeSpawnData, JobType, PlayerKillState, PvpCape,
};
use silkroad_protocol::ServerPacket;

pub(crate) fn visibility(
    mut query: Query<(Entity, &mut Visibility, &Position)>,
    lookup: Query<(Entity, &Position), With<Visibility>>,
) {
    for (entity, mut visibility, position) in query.iter_mut() {
        let entity: Entity = entity;
        let mut visibility: &mut Visibility = &mut visibility;
        let position: &Position = position;

        let entities_in_range = lookup
            .iter()
            .filter(|(other_entity, _)| {
                let other_entity: &Entity = other_entity;
                other_entity.id() != entity.id()
            })
            .filter(|(_, other_position)| {
                let other_position: &Position = other_position;
                let distance = position.location.0.distance(other_position.location.0);
                distance < visibility.visibility_radius
            })
            .map(|(entity, _)| entity)
            .collect();

        let removed: Vec<Entity> = visibility
            .entities_in_radius
            .difference(&entities_in_range)
            .map(|e| *e)
            .collect();
        let added: Vec<Entity> = visibility
            .entities_in_radius
            .symmetric_difference(&entities_in_range)
            .map(|e| *e)
            .collect();

        visibility.added_entities.extend(added);
        visibility.removed_entities.extend(removed);
    }
}

pub(crate) fn player_visibility_update(
    mut query: Query<(&Client, &mut Visibility)>,
    lookup: Query<(&Position, &NetworkedEntity, Option<&Player>)>,
) {
    for (client, mut visibility) in query.iter_mut() {
        let mut visibility: &mut Visibility = &mut visibility;

        for added in visibility.added_entities.iter() {
            if let Ok((pos, network_id, player_opt)) = lookup.get(*added) {
                let pos: &Position = pos;
                let network_id: &NetworkedEntity = network_id;
                let player_opt: Option<&Player> = player_opt;

                if let Some(player) = player_opt {
                    client.send(ServerPacket::EntitySpawn(EntitySpawn::new(
                        EntityTypeSpawnData::Character {
                            scale: 0,
                            berserk_level: 0,
                            pvp_cape: PvpCape::None,
                            beginner: true,
                            title: 0,
                            equipment: vec![],
                            avatar_items: vec![],
                            mask: None,
                            movement: pos.as_movement(),
                            entity_state: EntityState {
                                alive: AliveState::Alive,
                                unknown1: 0,
                                action_state: ActionState::None,
                                body_state: BodyState::None,
                                unknown2: 0,
                                walk_speed: 16.0,
                                run_speed: 50.0,
                                berserk_speed: 100.0,
                                active_buffs: vec![],
                            },
                            name: player.character.name.clone(),
                            job_type: JobType::None,
                            job_level: 0,
                            pk_state: PlayerKillState::None,
                            mounted: false,
                            in_combat: false,
                            active_scroll: ActiveScroll::None,
                            unknown2: 0,
                        },
                    )));
                }
            }
        }

        for removed in visibility.removed_entities.iter() {
            if let Ok((_, network_id, _)) = lookup.get(*removed) {
                client.send(ServerPacket::EntityDespawn(EntityDespawn::new(network_id.0)));
            }
        }

        visibility.added_entities.clear();
        visibility.removed_entities.clear();
    }
}
