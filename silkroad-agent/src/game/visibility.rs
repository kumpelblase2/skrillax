use crate::comp::drop::ItemDrop;
use crate::comp::monster::Monster;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{Client, EntityReference, GameEntity};
use bevy_ecs::prelude::*;
use cgmath::prelude::*;
use silkroad_protocol::inventory::CharacterSpawnItemData;
use silkroad_protocol::world::{
    ActionState, ActiveScroll, AliveState, BodyState, EntityMovementState, EntityState, EntityTypeSpawnData,
    GroupEntitySpawnData, GroupEntitySpawnEnd, GroupEntitySpawnStart, GroupSpawnDataContent, GroupSpawnType,
    GuildInformation, InteractOptions, JobType, MovementType, PlayerKillState, PvpCape,
};
use silkroad_protocol::ServerPacket;
use std::collections::HashSet;
use tracing::debug;

pub(crate) fn visibility_update(
    mut query: Query<(Entity, &mut Visibility, &Position)>,
    lookup: Query<(Entity, &Position, &GameEntity)>,
) {
    for (entity, mut visibility, position) in query.iter_mut() {
        let entities_in_range: HashSet<EntityReference> = lookup
            .iter()
            .filter(|(other_entity, _, _)| other_entity.id() != entity.id())
            .filter(|(_, other_position, _)| {
                let distance = position.location.0.distance(other_position.location.0);
                distance < visibility.visibility_radius
            })
            .map(|(entity, _, game_entity)| EntityReference(entity, *game_entity))
            .collect();

        let removed: Vec<EntityReference> = visibility
            .entities_in_radius
            .difference(&entities_in_range)
            .copied()
            .collect();
        let added: Vec<EntityReference> = entities_in_range
            .difference(&visibility.entities_in_radius)
            .copied()
            .collect();

        for reference in removed.iter() {
            debug!("Removed entity {:?} from visibility.", reference.0);
            visibility.entities_in_radius.remove(reference);
        }

        for reference in added.iter() {
            debug!("Added entity {:?} to visibility.", reference.0);
            visibility.entities_in_radius.insert(*reference);
        }

        visibility.added_entities.extend(added);
        visibility.removed_entities.extend(removed);
    }
}

pub(crate) fn player_visibility_update(
    mut query: Query<(&Client, &mut Visibility)>,
    lookup: Query<(&Position, Option<&Player>, Option<&Monster>, Option<&ItemDrop>)>,
) {
    for (client, mut visibility) in query.iter_mut() {
        let visibility: &mut Visibility = &mut visibility;

        let mut spawns = Vec::new();
        for reference in visibility.added_entities.iter() {
            let added = reference.0;
            let entity = reference.1;
            if let Ok((pos, player_opt, monster_opt, item_opt)) = lookup.get(added) {
                if let Some(player) = player_opt {
                    let items = player
                        .inventory
                        .items()
                        .map(|(_, item)| CharacterSpawnItemData {
                            item_id: item.ref_id as u32,
                            upgrade_level: item.upgrade_level,
                        })
                        .collect();
                    spawns.push(GroupSpawnDataContent::Spawn {
                        object_id: entity.ref_id,
                        data: EntityTypeSpawnData::Character {
                            unique_id: entity.unique_id,
                            scale: 0,
                            berserk_level: 0,
                            pvp_cape: PvpCape::None,
                            beginner: true,
                            title: 0,
                            inventory_size: player.inventory.size() as u8,
                            equipment: items,
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
                                siege_unknown: 0,
                            },
                            unknown3: [0; 9],
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
                            interaction_options: InteractOptions::talk(vec![5]),
                            rarity: monster.rarity,
                            unknown: 0,
                        },
                    ));
                } else if let Some(item) = item_opt {
                    debug!("Spawning gold {}.", entity.unique_id);
                    spawns.push(GroupSpawnDataContent::spawn(
                        entity.ref_id,
                        EntityTypeSpawnData::Gold {
                            amount: item.amount,
                            unique_id: entity.unique_id,
                            position: pos.as_protocol(),
                            owner: None,
                            rarity: 0,
                        },
                    ));
                }
            }
        }

        send_group_spawn_packet(client, GroupSpawnType::Spawn, spawns);

        let mut despawns = Vec::new();
        for reference in visibility.removed_entities.iter() {
            despawns.push(GroupSpawnDataContent::despawn(reference.1.unique_id));
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
