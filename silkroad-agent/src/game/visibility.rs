use crate::comp::monster::Monster;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity};
use bevy_ecs::prelude::*;
use bytes::{Bytes, BytesMut};
use cgmath::prelude::*;
use silkroad_protocol::world::{
    ActionState, ActiveScroll, AliveState, BodyState, EntityMovementState, EntityState, EntityTypeSpawnData,
    GroupEntitySpawnData, GroupEntitySpawnEnd, GroupEntitySpawnStart, GroupSpawnDataContent, GroupSpawnType,
    GuildInformation, InteractOptions, JobType, MovementType, PlayerKillState, PvpCape,
};
use silkroad_protocol::ServerPacket;
use tracing::debug;

static EMPTY_BYTES: [u8; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(crate) fn visibility_update(
    mut query: Query<(Entity, &mut Visibility, &Position)>,
    lookup: Query<(Entity, &Position), With<Visibility>>,
) {
    for (entity, mut visibility, position) in query.iter_mut() {
        let entity: Entity = entity;
        let visibility: &mut Visibility = &mut visibility;
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
            .copied()
            .collect();
        let added: Vec<Entity> = entities_in_range
            .difference(&visibility.entities_in_radius)
            .copied()
            .collect();

        for removed in removed.iter() {
            debug!("Removed entity {:?} from visibility.", removed);
            visibility.entities_in_radius.remove(removed);
        }

        for added in added.iter() {
            debug!("Added entity {:?} to visibility.", added);
            visibility.entities_in_radius.insert(*added);
        }

        visibility.added_entities.extend(added);
        visibility.removed_entities.extend(removed);
    }
}

pub(crate) fn player_visibility_update(
    mut query: Query<(&Client, &mut Visibility)>,
    lookup: Query<(&Position, &GameEntity, Option<&Player>, Option<&Monster>)>,
) {
    for (client, mut visibility) in query.iter_mut() {
        let visibility: &mut Visibility = &mut visibility;

        let mut spawns = Vec::new();
        for added in visibility.added_entities.iter() {
            if let Ok((pos, entity, player_opt, monster_opt)) = lookup.get(*added) {
                let pos: &Position = pos;
                let entity: &GameEntity = entity;
                let player_opt: Option<&Player> = player_opt;
                let monster_opt: Option<&Monster> = monster_opt;

                if let Some(player) = player_opt {
                    spawns.push(GroupSpawnDataContent::Spawn {
                        object_id: entity.ref_id,
                        data: EntityTypeSpawnData::Character {
                            unique_id: entity.unique_id,
                            scale: 0,
                            berserk_level: 0,
                            pvp_cape: PvpCape::None,
                            beginner: true,
                            title: 0,
                            inventory_size: 45,
                            equipment: vec![],
                            avatar_inventory_size: 5,
                            avatar_items: vec![],
                            mask: None,
                            position: pos.as_protocol(),
                            movement: EntityMovementState::standing(MovementType::Walking, 0, pos.rotation.into()),
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
                            pk_state: PlayerKillState::None,
                            mounted: false,
                            in_combat: false,
                            active_scroll: ActiveScroll::None,
                            unknown2: 0,
                            guild: GuildInformation {
                                name: "".to_string(),
                                id: 0,
                                member: "".to_string(),
                                last_icon_rev: 0,
                                union_id: 0,
                                last_union_icon_rev: 0,
                                is_friendly: 0,
                                siege_unkown: 0,
                            },
                            unknown3: Bytes::from(&EMPTY_BYTES[..]),
                            equipment_cooldown: false,
                            unknown4: 0,
                        },
                    });
                } else if let Some(monster) = monster_opt {
                    debug!("Spawning monster {}.", entity.unique_id);
                    spawns.push(GroupSpawnDataContent::spawn(
                        entity.ref_id,
                        EntityTypeSpawnData::Monster {
                            unique_id: entity.unique_id,
                            position: pos.as_protocol(),
                            movement: pos.as_movement(),
                            entity_state: EntityState {
                                alive: AliveState::Alive,
                                unknown1: 0,
                                action_state: ActionState::None,
                                body_state: BodyState::None,
                                unknown2: 0,
                                walk_speed: 16.0,
                                run_speed: 40.0,
                                berserk_speed: 80.0,
                                active_buffs: vec![],
                            },
                            interaction_options: InteractOptions::None,
                            rarity: monster.rarity,
                            unknown: 0,
                        },
                    ));
                }
            }
        }

        send_group_spawn_packet(client, GroupSpawnType::Spawn, spawns);

        let mut despawns = Vec::new();
        for removed in visibility.removed_entities.iter() {
            if let Ok((_, entity, _, _)) = lookup.get(*removed) {
                despawns.push(GroupSpawnDataContent::despawn(entity.unique_id));
            }
        }

        send_group_spawn_packet(client, GroupSpawnType::Despawn, despawns);

        visibility.added_entities.clear();
        visibility.removed_entities.clear();
    }
}

fn send_group_spawn_packet(client: &Client, mode: GroupSpawnType, spawns: Vec<GroupSpawnDataContent>) {
    if !spawns.is_empty() {
        client.send(ServerPacket::GroupEntitySpawnStart(GroupEntitySpawnStart::new(
            mode,
            spawns.len() as u16,
        )));
        client.send(ServerPacket::GroupEntitySpawnData(GroupEntitySpawnData::new(spawns)));
        client.send(ServerPacket::GroupEntitySpawnEnd(GroupEntitySpawnEnd));
    }
}
