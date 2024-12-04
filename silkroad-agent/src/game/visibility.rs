use crate::agent::Agent;
use crate::comp::drop::Drop;
use crate::comp::inventory::PlayerInventory;
use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::npc::NPC;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::visibility::{Invisible, Visibility};
use crate::comp::{EntityReference, GameEntity};
use crate::game::player_activity::PlayerActivity;
use bevy_ecs::prelude::*;
use cgmath::num_traits::Pow;
use silkroad_data::DataEntry;
use silkroad_definitions::Region;
use silkroad_game_base::ItemTypeData;
use silkroad_navmesh::region::GridRegion;
use silkroad_protocol::community::GuildInformation;
use silkroad_protocol::inventory::CharacterSpawnItemData;
use silkroad_protocol::spawn::{
    DroppedItemSource, EntityTypeSpawnData, GroupEntitySpawnData, GroupEntitySpawnEnd, GroupEntitySpawnStart,
    GroupSpawnDataContent, GroupSpawnType, ItemSpawnData,
};
use silkroad_protocol::world::{
    ActionState, ActiveScroll, AliveState, BodyState, EntityState, InteractOptions, JobType, PlayerKillState, PvpCape,
};
use std::collections::{BTreeMap, HashSet};
use tracing::{instrument, trace};

static EMPTY_VEC: Vec<(Entity, &Position, &GameEntity)> = vec![];

#[instrument(level = "trace", skip_all)]
pub(crate) fn visibility_update(
    activity: Res<PlayerActivity>,
    mut query: Query<(Entity, &GameEntity, &mut Visibility, &Position)>,
    lookup: Query<(Entity, &Position, &GameEntity)>,
) {
    let grouped = lookup.iter().fold(
        BTreeMap::new(),
        |mut acc: BTreeMap<Region, Vec<(Entity, &Position, &GameEntity)>>, (entity, pos, game_entity)| {
            acc.entry(pos.position().region())
                .or_default()
                .push((entity, pos, game_entity));
            acc
        },
    );

    query.par_iter_mut().for_each(|(entity, game_entity, mut visibility, position)| {
        let my_region = position.position().region();
        let close_regions = my_region.with_grid_neighbours();
        if close_regions.iter().any(|region| activity.is_region_active(region)) {
            let entities_in_range: HashSet<EntityReference> = close_regions
                    .iter()
                    .flat_map(|region| grouped.get(region).unwrap_or(&EMPTY_VEC))
                    .filter(|(other_entity, _, _)| other_entity.index() != entity.index())
                    .filter(|(_, other_position, _)| {
                        position.distance_to(other_position) < (visibility.visibility_radius.pow(2))
                    })
                .map(|(entity, _, game_entity)| EntityReference(*entity, **game_entity))
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
                trace!(player = ?game_entity.unique_id, "Removed entity {} from visibility.", reference.1.unique_id);
                visibility.entities_in_radius.remove(reference);
            }

            for reference in added.iter() {
                trace!(player = ?game_entity.unique_id, "Added entity {} to visibility.", reference.1.unique_id);
                visibility.entities_in_radius.insert(*reference);
            }

            visibility.added_entities.extend(added);
            visibility.removed_entities.extend(removed);
        }
    });
}

pub(crate) fn player_visibility_update(
    mut query: Query<(&Client, &GameEntity, &mut Visibility)>,
    lookup: Query<
        (
            &Position,
            Option<&PlayerInventory>,
            Option<&Agent>,
            Option<&Player>,
            Option<&Monster>,
            Option<&Drop>,
            Option<&NPC>,
        ),
        Without<Invisible>,
    >,
) {
    for (client, player, mut visibility) in query.iter_mut() {
        let mut spawns = Vec::new();
        for reference in visibility.added_entities.iter() {
            let added = reference.0;
            let entity = reference.1;
            if let Ok((pos, inventory_opt, agent_opt, player_opt, monster_opt, item_opt, npc_opt)) = lookup.get(added) {
                if let Some(player) = player_opt {
                    let agent = agent_opt.unwrap();
                    let items = inventory_opt
                        .map(|inv| {
                            inv.equipment_items()
                                .map(|(_, item)| CharacterSpawnItemData {
                                    item_id: item.reference.ref_id(),
                                    upgrade_level: item.upgrade_level(),
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    spawns.push(GroupSpawnDataContent::Spawn {
                        object_id: entity.ref_id,
                        data: EntityTypeSpawnData::character(
                            player.character.scale,
                            0,
                            PvpCape::None,
                            player.character.beginner_mark,
                            0,
                            inventory_opt.map(|inv| inv.size() as u8).unwrap_or(0),
                            items,
                            5,
                            vec![],
                            None,
                            entity.unique_id,
                            pos.as_protocol(),
                            pos.as_standing(),
                            entity_state_from_agent(agent),
                            player.character.name.clone(),
                            JobType::None,
                            false,
                            false,
                            ActiveScroll::None,
                            GuildInformation::none(),
                            false,
                            PlayerKillState::None,
                        ),
                    });
                } else if let Some(monster) = monster_opt {
                    let agent = agent_opt.unwrap();
                    spawns.push(GroupSpawnDataContent::spawn(
                        entity.ref_id,
                        EntityTypeSpawnData::Monster {
                            unique_id: entity.unique_id,
                            position: pos.as_protocol(),
                            movement: pos.as_movement(),
                            entity_state: entity_state_from_agent(agent),
                            // Somehow doesn't matter right now *shrug*
                            interaction_options: InteractOptions::None,
                            rarity: monster.rarity,
                            unknown: 0,
                        },
                    ));
                } else if let Some(drop) = item_opt {
                    let spawn_data = spawndata_from_item(entity, pos, drop, player);
                    spawns.push(GroupSpawnDataContent::Spawn {
                        object_id: entity.ref_id,
                        data: EntityTypeSpawnData::Item(spawn_data),
                    });
                } else if let Some(_) = npc_opt {
                    let agent = agent_opt.unwrap();
                    spawns.push(GroupSpawnDataContent::spawn(
                        entity.ref_id,
                        EntityTypeSpawnData::NPC {
                            unique_id: entity.unique_id,
                            position: pos.as_protocol(),
                            movement: pos.as_standing(),
                            entity_state: entity_state_from_agent(agent),
                            interaction_options: InteractOptions::None,
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

fn entity_state_from_agent(agent: &Agent) -> EntityState {
    EntityState {
        alive: AliveState::Alive,
        unknown1: 0,
        action_state: ActionState::None,
        body_state: BodyState::None,
        unknown2: 0,
        walk_speed: agent.walking_speed,
        run_speed: agent.running_speed,
        berserk_speed: agent.berserk_speed,
        active_buffs: vec![],
    }
}

fn spawndata_from_item(entity: GameEntity, pos: &Position, drop: &Drop, for_player: &GameEntity) -> ItemSpawnData {
    match drop.item.type_data {
        ItemTypeData::Equipment { upgrade_level } => ItemSpawnData::Equipment {
            upgrade: upgrade_level,
            unique_id: entity.unique_id,
            position: pos.as_protocol(),
            owner: drop
                .owner
                .map(|owner| owner.1.unique_id)
                .filter(|id| *id != for_player.unique_id),
            rarity: 0,
            source: DroppedItemSource::None,
            source_id: 0,
        },
        ItemTypeData::COS | ItemTypeData::Consumable { .. } => ItemSpawnData::Consumable {
            unique_id: entity.unique_id,
            position: pos.as_protocol(),
            owner: drop
                .owner
                .map(|owner| owner.1.unique_id)
                .filter(|id| *id != for_player.unique_id),
            rarity: 0,
            source: DroppedItemSource::None,
            source_id: 0,
        },
        ItemTypeData::Gold { amount } => ItemSpawnData::Gold {
            amount,
            unique_id: entity.unique_id,
            position: pos.as_protocol(),
            owner: drop
                .owner
                .map(|owner| owner.1.unique_id)
                .filter(|id| *id != for_player.unique_id),
            rarity: 0,
        },
    }
}

pub(crate) fn clear_visibility(mut query: Query<&mut Visibility, Without<Player>>) {
    for mut visibility in query.iter_mut() {
        visibility.added_entities.clear();
        visibility.removed_entities.clear();
    }
}

fn send_group_spawn_packet(client: &Client, mode: GroupSpawnType, spawns: Vec<GroupSpawnDataContent>) {
    if !spawns.is_empty() {
        client.send(GroupEntitySpawnStart::new(mode, spawns.len() as u16));
        client.send(GroupEntitySpawnData::new(spawns));
        client.send(GroupEntitySpawnEnd);
    }
}
