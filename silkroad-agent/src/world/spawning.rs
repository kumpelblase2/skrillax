use crate::agent::component::{Agent, MovementState};
use crate::agent::goal::GoalTracker;
use crate::agent::state::{AgentStateQueue, Dead};
use crate::comp::damage::DamageReceiver;
use crate::comp::monster::{Monster, MonsterAiBundle, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::npc::NpcBundle;
use crate::comp::pos::Position;
use crate::comp::spawner::Spawner;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::config::GameConfig;
use crate::ext::{EntityIdPool, Navmesh, NpcPositionList};
use crate::game::player_activity::PlayerActivity;
use crate::world::WorldData;
use bevy::prelude::*;
use cgmath::Vector3;
use id_pool::IdPool;
use rand::Rng;
use silkroad_data::characterdata::RefCharacterData;
use silkroad_data::DataEntry;
use silkroad_definitions::rarity::EntityRarityType;
use silkroad_definitions::type_id::{ObjectEntity, ObjectMonster, ObjectNonPlayer, ObjectType};
use silkroad_definitions::Region;
use silkroad_game_base::{GlobalLocation, Heading, LocalPosition, Vector2Ext};
use silkroad_navmesh::region::GridRegion;
use silkroad_navmesh::GlobalNavmesh;
use std::cmp::min;
use std::collections::HashSet;
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
        ) && character_data.rarity == EntityRarityType::Normal
        {
            let pos = LocalPosition(spawn.region.into(), Vector3::new(spawn.x, spawn.y, spawn.z)).to_global();
            let position = Position::new(pos, Heading(0.0));
            commands.spawn((Spawner::new(&settings.spawner, character_data), position));
        }
    }
}

pub(crate) fn spawn_monsters(
    mut query: Query<(Entity, &mut Spawner, &Position)>,
    mut commands: Commands,
    activity: Res<PlayerActivity>,
    navmesh: Res<Navmesh>,
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
        let should_be_active = active_regions.contains(&position.position().region());
        if !spawner.active && should_be_active {
            trace!(spawner = ?entity, "Activating spawner");
            activate_spawner(entity, &mut spawner, position, &mut commands, &navmesh, &mut id_pool);
        } else if spawner.active {
            if !should_be_active {
                trace!(spawner = ?entity, "Deactivating spawner");
                deactivate_spawner(entity, &mut spawner, &mut commands, &despawn_query);
            } else if spawner.should_spawn(delta) {
                let empty_spots = spawner.available_spots();
                let max_spawn = min(empty_spots, 3); // Spawn at most 3 at once
                let to_spawn = rand::rng().random_range(1..=max_spawn);

                let spawned_amount = spawn_n_monsters(
                    entity,
                    &mut commands,
                    &navmesh,
                    &mut id_pool,
                    &mut spawner,
                    position,
                    to_spawn,
                );
                spawner.increase_alive_by(spawned_amount);
            }
        }
    }
}

fn spawn_n_monsters(
    spawner_entity: Entity,
    commands: &mut Commands,
    navmesh: &GlobalNavmesh,
    id_pool: &mut IdPool,
    spawner: &mut Spawner,
    position: &Position,
    to_spawn: usize,
) -> usize {
    (0..to_spawn)
        .map(|_| generate_position(position, spawner.radius))
        .filter_map(|loc| to_position(loc, navmesh))
        .for_each(|pos| {
            spawn_monster(
                spawner_entity,
                spawner.reference,
                id_pool.request_id().unwrap(),
                pos,
                commands,
            )
        });

    to_spawn
}

fn activate_spawner(
    entity: Entity,
    spawner: &mut Spawner,
    position: &Position,
    commands: &mut Commands,
    navmesh: &GlobalNavmesh,
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
    spawner.increase_alive_by(spawned);
    spawner.active = true;
}

fn deactivate_spawner(
    entity: Entity,
    spawner: &mut Spawner,
    commands: &mut Commands,
    despawn_query: &Query<(Entity, &SpawnedBy)>,
) {
    for (spawned_entity, spawned_by) in despawn_query.iter() {
        match spawned_by {
            SpawnedBy::Spawner(e) if *e == entity => {
                commands.entity(spawned_entity).despawn();
            },
            _ => {},
        }
    }
    spawner.deactivate();
}

fn generate_position(source_position: &Position, radius: f32) -> GlobalLocation {
    let vec = source_position.location().0.random_in_radius(radius);
    GlobalLocation(vec)
}

fn to_position(location: GlobalLocation, navmesh: &GlobalNavmesh) -> Option<Position> {
    let local = location.to_local();
    let navmesh = navmesh.mesh_for(local.0)?;
    let height = navmesh
        .heightmap()
        .height_at_position(local.1.x, local.1.y)
        .expect("Location should be inside region.");
    let pos = location.with_y(height);
    let heading = Heading(rand::rng().random_range(0..360) as f32);
    Some(Position::new(pos, heading))
}

fn spawn_monster(
    spawner: Entity,
    reference: &RefCharacterData,
    unique_id: u32,
    target_location: Position,
    cmd: &mut Commands,
) {
    let spawn_center = target_location.location();
    let bundle = MonsterBundle {
        monster: Monster {
            target: None,
            rarity: EntityRarityType::Normal.into(),
        },
        health: Health::new(reference.hp),
        position: target_location,
        entity: GameEntity {
            ref_id: reference.ref_id(),
            unique_id,
        },
        visibility: Visibility::with_radius(500.0),
        spawner: SpawnedBy::Spawner(spawner),
        navigation: Agent::default(),
        state_queue: AgentStateQueue::default(),
        movement_state: MovementState::default_monster(),
        damage_receiver: DamageReceiver::default(),
    };

    let ai_bundle = MonsterAiBundle {
        stroll: RandomStroll::new(spawn_center, 300.0, 10..60),
        goal: GoalTracker::default(),
    };

    cmd.spawn((bundle, ai_bundle));
}

pub(crate) fn collect_monster_deaths(
    mut spawner_query: Query<&mut Spawner>,
    entity_query: Query<&SpawnedBy, (With<Monster>, Added<Dead>)>,
) {
    for spawned_by in entity_query.iter() {
        let SpawnedBy::Spawner(e) = spawned_by else {
            continue;
        };

        let Ok(mut spawner) = spawner_query.get_mut(*e) else {
            continue;
        };

        spawner.decrease_alive();
    }
}
