use crate::comp::player::Player;
use crate::comp::GameEntity;
use bevy_ecs::entity::Entities;
use bevy_ecs::prelude::*;
use id_pool::IdPool;
use std::collections::HashMap;
use tracing::debug;

#[derive(Default)]
pub struct EntityLookup {
    player_map: HashMap<String, Entity>,
    id_map: HashMap<u32, Entity>,
}

impl EntityLookup {
    pub fn add_player(&mut self, name: String, entity: Entity, id: u32) {
        self.player_map.insert(name, entity);
        self.id_map.insert(id, entity);
    }

    pub fn add_entity(&mut self, entity_id: u32, entity: Entity) {
        self.id_map.insert(entity_id, entity);
    }

    pub fn get_entity_for_name(&self, name: &String) -> Option<Entity> {
        self.player_map.get(name).copied()
    }

    pub fn get_entity_for_id(&self, id: u32) -> Option<Entity> {
        self.id_map.get(&id).copied()
    }
}

pub(crate) fn collect_entities(
    mut lookup: ResMut<EntityLookup>,
    query: Query<(Entity, &GameEntity, Option<&Player>), Added<GameEntity>>,
) {
    for (entity, game_entity, player_opt) in query.iter() {
        if let Some(player) = player_opt {
            lookup.add_player(player.character.name.clone(), entity, game_entity.unique_id);
        } else {
            lookup.add_entity(game_entity.unique_id, entity);
        }
    }
}

pub fn maintain_entities(mut lookup: ResMut<EntityLookup>, mut id_pool: ResMut<IdPool>, entities: &Entities) {
    let before_player_count = lookup.player_map.len();
    lookup.player_map.retain(|_, entity| entities.contains(*entity));
    let after_player_count = lookup.player_map.len();
    if before_player_count != after_player_count {
        debug!(
            "Updated player count lookup: {} -> {}",
            before_player_count, after_player_count
        );
    }
    let removed_entities: Vec<u32> = lookup
        .id_map
        .iter()
        .filter(|(_, entity)| !entities.contains(**entity))
        .map(|(id, _)| *id)
        .collect();

    for id in removed_entities {
        lookup.id_map.remove(&id);
        let _ = id_pool.return_id(id);
    }
}
