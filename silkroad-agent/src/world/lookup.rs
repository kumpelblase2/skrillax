use bevy_ecs::entity::{Entities, Entity};
use bevy_ecs::system::ResMut;
use id_pool::IdPool;
use std::collections::HashMap;
use tracing::debug;

pub struct EntityLookup {
    player_map: HashMap<String, Entity>,
    id_map: HashMap<u32, Entity>,
}

impl EntityLookup {
    pub fn add_player(&mut self, name: String, entity: Entity, id: u32) {
        self.player_map.insert(name.clone(), entity);
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
    pub fn new() -> Self {
        Self {
            player_map: HashMap::new(),
            id_map: HashMap::new(),
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
