use bevy_ecs::entity::{Entities, Entity};
use bevy_ecs::system::ResMut;
use crossbeam_channel::after;
use std::collections::HashMap;
use tracing::debug;

pub(crate) struct EntityLookup {
    player_map: HashMap<String, Entity>,
    id_map: HashMap<u32, Entity>,
}

impl EntityLookup {
    pub fn add_player(&mut self, name: String, entity: Entity, id: u32) {
        self.player_map.insert(name.clone(), entity);
        self.id_map.insert(id, entity);
    }

    pub fn add_monster(&mut self, monster_id: u32, entity: Entity) {
        self.id_map.insert(monster_id, entity);
    }

    pub fn get_entity_for_name(&self, name: &String) -> Option<&Entity> {
        self.player_map.get(name)
    }

    pub fn get_entity_for_id(&self, id: u32) -> Option<&Entity> {
        self.id_map.get(&id)
    }
    pub fn new() -> Self {
        Self {
            player_map: HashMap::new(),
            id_map: HashMap::new(),
        }
    }
}

pub(crate) fn maintain_entities(mut lookup: ResMut<EntityLookup>, entities: &Entities) {
    let before_player_count = lookup.player_map.len();
    lookup.player_map.retain(|_, entity| entities.contains(*entity));
    let after_player_count = lookup.player_map.len();
    if before_player_count != after_player_count {
        debug!(
            "Updated player count lookup: {} -> {}",
            before_player_count, after_player_count
        );
    }
    lookup.id_map.retain(|_, entity| entities.contains(*entity));
}
