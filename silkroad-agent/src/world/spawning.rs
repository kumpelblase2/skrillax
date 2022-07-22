use crate::comp::monster::{Monster, MonsterBundle, Spawner};
use crate::comp::npc::{NpcBundle, NPC};
use crate::comp::pos::{GlobalLocation, Heading, LocalPosition, Position};
use crate::comp::visibility::Visibility;
use crate::comp::GameEntity;
use crate::math::random_point_in_circle;
use crate::world::EntityLookup;
use crate::GameSettings;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use cgmath::Vector3;
use id_pool::IdPool;
use pk2::Pk2;
use rand::{random, Rng};
use silkroad_data::characterdata::CharacterMap;
use silkroad_data::npc_pos::NpcPosition;
use silkroad_data::type_id::{ObjectEntity, ObjectMonster, ObjectNonPlayer, ObjectType};
use silkroad_navmesh::NavmeshLoader;
use silkroad_protocol::world::EntityRarity;
use std::cmp::min;
use std::time::{Duration, Instant};

pub fn spawn_npcs(
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
    mut query: Query<(&mut Spawner, &Position)>,
    mut commands: Commands,
    time: Res<Time>,
    mut navmesh: ResMut<NavmeshLoader<Pk2>>,
    mut lookup: ResMut<EntityLookup>,
    mut id_pool: ResMut<IdPool>,
) {
    let current_time = if let Some(current) = time.last_update() {
        current
    } else {
        return;
    };

    for (mut spawner, position) in query.iter_mut() {
        if spawner.has_spots_available()
            && current_time.duration_since(spawner.last_spawn_check) > Duration::from_secs(1)
        {
            spawner.last_spawn_check = current_time;
            if random::<f32>() > 0.8 {
                let empty_spots = spawner.target_amount - spawner.current_amount;
                let max_spawn = min(empty_spots, 3); // Spawn at most 3 at once
                let to_spawn = rand::thread_rng().gen_range(1..=max_spawn);
                let spawned = (0..to_spawn)
                    .map(|_| generate_position(position, spawner.radius))
                    .filter_map(|loc| to_position(loc, &mut navmesh))
                    .map(|pos| spawn_monster(spawner.ref_id, id_pool.request_id().unwrap(), pos))
                    .collect::<Vec<MonsterBundle>>();

                let spawned_amount = spawned.len();
                for bundle in spawned {
                    let unique_id = bundle.entity.unique_id;
                    let spawned = commands.spawn().insert_bundle(bundle).id();
                    lookup.add_entity(unique_id, spawned);
                }
                spawner.current_amount += spawned_amount;
            }
        }
    }
}

fn generate_position(source_position: &Position, radius: f32) -> GlobalLocation {
    let vec = random_point_in_circle(source_position.location.to_location().0, radius);
    GlobalLocation(vec)
}

fn to_position(location: GlobalLocation, mut navmesh: &mut NavmeshLoader<Pk2>) -> Option<Position> {
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

fn spawn_monster(ref_id: u32, unique_id: u32, target_location: Position) -> MonsterBundle {
    MonsterBundle {
        monster: Monster {
            target: None,
            rarity: EntityRarity::Normal,
        },
        position: target_location,
        entity: GameEntity { ref_id, unique_id },
    }
}
