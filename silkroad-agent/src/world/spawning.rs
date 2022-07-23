use crate::comp::monster::{Monster, MonsterBundle, RandomStroll, SpawnedBy, Spawner};
use crate::comp::npc::NpcBundle;
use crate::comp::player::{Agent, MovementState};
use crate::comp::pos::{GlobalLocation, Heading, LocalPosition, Position};
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::game::player_activity::PlayerActivity;
use crate::math::random_point_in_circle;
use crate::GameSettings;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use id_pool::IdPool;
use pk2::Pk2;
use rand::{random, Rng};
use silkroad_data::characterdata::CharacterMap;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_data::type_id::{ObjectEntity, ObjectMonster, ObjectNonPlayer, ObjectType};
use silkroad_navmesh::region::Region;
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::EntityRarity;
use std::cmp::min;
use std::time::{Duration, Instant};
use tracing::trace;

pub(crate) fn spawn_npcs(
    npc_spawns: Res<Vec<NpcPosition>>,
    character_data: Res<CharacterMap>,
    settings: Res<GameSettings>,
    mut commands: Commands,
    mut id_pool: ResMut<IdPool>,
) {
    for spawn in npc_spawns.iter() {
        let character_data = character_data
            .find_id(spawn.npc_id)
            .expect("Could not find character data for NPC to spawn.");
        let type_id =
            ObjectType::from_type_id(&character_data.type_id).expect("Could not create type id from type 4-tuple.");
        if matches!(
            type_id,
            ObjectType::Entity(ObjectEntity::NonPlayer(ObjectNonPlayer::NPC(_)))
        ) {
            commands.spawn().insert_bundle(NpcBundle::new(
                id_pool.request_id().expect("Should have ID available for NPC"),
                spawn.npc_id,
                LocalPosition(spawn.region.into(), Vector3::new(spawn.x, spawn.y, spawn.z)),
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
            commands
                .spawn()
                .insert(Spawner::new(&settings.spawn_settings, spawn.npc_id))
                .insert(position);
        }
    }
}

pub(crate) fn spawn_monsters(
    mut query: Query<(Entity, &mut Spawner, &Position)>,
    mut commands: Commands,
    activity: Res<PlayerActivity>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    mut id_pool: ResMut<IdPool>,
    despawn_query: Query<(Entity, &SpawnedBy)>,
) {
    let current_time = Instant::now();

    let mut active_regions = activity.set.clone();
    activity
        .set
        .iter()
        .flat_map(|r| Region::from(*r).neighbours())
        .for_each(|r| {
            let _ = active_regions.insert(r.id());
        });

    for (entity, mut spawner, position) in query.iter_mut() {
        let should_be_active = active_regions.contains(&position.location.region().id());
        if !spawner.active && should_be_active {
            trace!("Activating spawner {:?}", entity);
            activate_spawner(
                entity,
                &mut spawner,
                position,
                &mut commands,
                &mut navmesh,
                &mut id_pool,
            );
        } else if spawner.active && !should_be_active {
            trace!("Deactivating spawner {:?}", entity);
            deactivate_spawner(entity, &mut spawner, &mut commands, &despawn_query);
        } else if spawner.active {
            if spawner.has_spots_available()
                && current_time.duration_since(spawner.last_spawn_check) > Duration::from_secs(1)
            {
                spawner.last_spawn_check = current_time;
                if random::<f32>() > 0.5 {
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
}

fn spawn_n_monsters(
    spawner_entity: Entity,
    commands: &mut Commands,
    mut navmesh: &mut NavmeshLoader<Pk2>,
    id_pool: &mut IdPool,
    spawner: &mut Spawner,
    position: &Position,
    to_spawn: usize,
) -> usize {
    let spawned = (0..to_spawn)
        .map(|_| generate_position(position, spawner.radius))
        .filter_map(|loc| to_position(loc, &mut navmesh))
        .map(|pos| spawn_monster(spawner_entity, spawner.ref_id, id_pool.request_id().unwrap(), 54, pos))
        .collect::<Vec<MonsterBundle>>();

    let spawned_amount = spawned.len();
    for bundle in spawned {
        commands.spawn().insert_bundle(bundle);
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
    let vec = random_point_in_circle(source_position.location.to_location().0, radius);
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
        navigation: Agent {
            movement_speed: 16.0,
            movement_state: MovementState::Standing,
            movement_target: None,
        },
        sync: Synchronize::default(),
        stroll: RandomStroll::new(spawn_center, 100.0, Duration::from_secs(2)),
    }
}
