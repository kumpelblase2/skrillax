use crate::agent::states::StateTransitionQueue;
use crate::agent::Agent;
use crate::comp::monster::{Monster, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::npc::NpcBundle;
use crate::comp::pos::Position;
use crate::comp::spawner::Spawner;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::config::GameConfig;
use crate::ext::{EntityIdPool, Navmesh, NpcPositionList};
use crate::game::player_activity::PlayerActivity;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use cgmath::Vector3;
use id_pool::IdPool;
use pk2::Pk2;
use rand::Rng;
use silkroad_data::type_id::{ObjectEntity, ObjectMonster, ObjectNonPlayer, ObjectType};
use silkroad_game_base::{GlobalLocation, Heading, LocalPosition, Vector2Ext};
use silkroad_navmesh::region::Region;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::EntityRarity;
use std::cmp::min;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tracing::trace;

pub(crate) fn spawn_npcs(
    npc_spawns: Res<NpcPositionList>,
    settings: Res<GameConfig>,
    mut commands: Commands,
    mut id_pool: ResMut<EntityIdPool>,
) {
    for spawn in npc_spawns.iter() {
        let character_data = WorldData::characters()
            .find_id(spawn.npc_id)
            .expect("Could not find character data for NPC to spawn.");
        let type_id = ObjectType::from_type_id(&character_data.common.type_id)
            .expect("Could not create type id from type 4-tuple.");
        if matches!(
            type_id,
            ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(_)))
        ) {
            commands.spawn(NpcBundle::new(
                id_pool.request_id().expect("Should have ID available for NPC"),
                spawn.npc_id,
                LocalPosition(spawn.region.into(), Vector3::new(spawn.x, spawn.y, spawn.z)),
                Agent::from_character_data(character_data),
            ));
        } else if matches!(
            type_id,
            ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::Monster(
                ObjectMonster::General
            )))
        ) {
            let position = Position {
                location: LocalPosition(spawn.region.into(), Vector3::new(spawn.x, spawn.y, spawn.z)).to_global(),
                rotation: Heading(0.0),
            };
            commands.spawn((Spawner::new(&settings.spawner, spawn.npc_id), position));
        }
    }
}

pub(crate) fn spawn_monsters(
    mut query: Query<(Entity, &mut Spawner, &Position)>,
    mut commands: Commands,
    activity: Res<PlayerActivity>,
    mut navmesh: ResMut<Navmesh>,
    mut id_pool: ResMut<EntityIdPool>,
    time: Res<Time>,
    despawn_query: Query<(Entity, &SpawnedBy)>,
) {
    let delta = time.delta();
    let active_regions: HashSet<Region> = activity
        .active_regions()
        .flat_map(|region| region.with_grid_neighbours())
        .collect();

    for (entity, mut spawner, position) in query.iter_mut() {
        let should_be_active = active_regions.contains(&position.location.region());
        if !spawner.active && should_be_active {
            trace!(spawner = ?entity, "Activating spawner");
            activate_spawner(
                entity,
                &mut spawner,
                position,
                &mut commands,
                &mut navmesh,
                &mut id_pool,
            );
        } else if spawner.active && !should_be_active {
            trace!(spawner = ?entity, "Deactivating spawner");
            deactivate_spawner(entity, &mut spawner, &mut commands, &despawn_query);
        } else if spawner.active {
            if spawner.has_spots_available() && spawner.should_spawn(delta) {
                let empty_spots = spawner.target_amount - spawner.current_amount;
                let max_spawn = min(empty_spots, 3); // Spawn at most 3 at once
                let to_spawn = rand::thread_rng().gen_range(1..=max_spawn);

                let spawned_amount = spawn_n_monsters(
                    entity,
                    &mut commands,
                    &mut navmesh,
                    &mut id_pool,
                    &mut spawner,
                    position,
                    to_spawn,
                );
                spawner.current_amount += spawned_amount;
            }
        }
    }
}

fn spawn_n_monsters(
    spawner_entity: Entity,
    commands: &mut Commands,
    navmesh: &mut NavmeshLoader<Pk2>,
    id_pool: &mut IdPool,
    spawner: &mut Spawner,
    position: &Position,
    to_spawn: usize,
) -> usize {
    let spawned = (0..to_spawn)
        .map(|_| generate_position(position, spawner.radius))
        .filter_map(|loc| to_position(loc, navmesh))
        .map(|pos| spawn_monster(spawner_entity, spawner.ref_id, id_pool.request_id().unwrap(), 54, pos))
        .collect::<Vec<MonsterBundle>>();

    let spawned_amount = spawned.len();
    for bundle in spawned {
        commands.spawn(bundle);
    }
    spawned_amount
}

fn activate_spawner(
    entity: Entity,
    spawner: &mut Spawner,
    position: &Position,
    commands: &mut Commands,
    navmesh: &mut NavmeshLoader<Pk2>,
    id_pool: &mut IdPool,
) {
    let spawned = spawn_n_monsters(
        entity,
        commands,
        navmesh,
        id_pool,
        spawner,
        position,
        spawner.target_amount,
    );
    spawner.active = true;
    spawner.current_amount = spawned;
}

fn deactivate_spawner(
    entity: Entity,
    spawner: &mut Spawner,
    commands: &mut Commands,
    despawn_query: &Query<(Entity, &SpawnedBy)>,
) {
    for (spawned_entity, spawned_by) in despawn_query.iter() {
        if spawned_by.spawner == entity {
            commands.entity(spawned_entity).despawn();
        }
    }
    spawner.current_amount = 0;
    spawner.active = false;
}

fn generate_position(source_position: &Position, radius: f32) -> GlobalLocation {
    let vec = source_position.location.to_location().0.random_in_radius(radius);
    GlobalLocation(vec)
}

fn to_position(location: GlobalLocation, navmesh: &mut NavmeshLoader<Pk2>) -> Option<Position> {
    let local = location.to_local();
    let navmesh = if let Ok(mesh) = navmesh.load_navmesh(local.0) {
        mesh
    } else {
        return None;
    };
    let height = navmesh
        .heightmap()
        .height_at_position(local.1.x, local.1.y)
        .expect("Location should be inside region.");
    let pos = location.with_y(height);
    let heading = Heading(rand::thread_rng().gen_range(0..360) as f32);
    Some(Position {
        location: pos,
        rotation: heading,
    })
}

fn spawn_monster(
    spawner: Entity,
    ref_id: u32,
    unique_id: u32,
    health: u32,
    target_location: Position,
) -> MonsterBundle {
    let spawn_center = target_location.location.to_location();
    MonsterBundle {
        monster: Monster {
            target: None,
            rarity: EntityRarity::Normal,
        },
        health: Health::new(health),
        position: target_location,
        entity: GameEntity { ref_id, unique_id },
        visibility: Visibility::with_radius(100.0),
        spawner: SpawnedBy { spawner },
        navigation: Agent::default(),
        sync: Synchronize::default(),
        stroll: RandomStroll::new(spawn_center, 100.0, Duration::from_secs(2)),
        state_queue: StateTransitionQueue::default(),
    }
}
