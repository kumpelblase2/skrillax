use crate::comp::player::Player;
use crate::comp::GameEntity;
use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use std::collections::HashMap;
use tracing::debug;

#[derive(Default, Resource)]
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

pub fn maintain_entities(mut lookup: ResMut<EntityLookup>, entities: &Entities) {
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

    // Runtime IDs have no generation on the wire. Reusing one could make a
    // delayed client action resolve to an unrelated newly spawned entity, so
    // allocation remains monotonic for the lifetime of this server process.
    for id in removed_entities {
        lookup.id_map.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::EntityIdPool;

    #[test]
    fn despawned_runtime_ids_are_not_reused() {
        let mut app = App::new();
        app.init_resource::<EntityLookup>();
        app.init_resource::<EntityIdPool>();
        app.add_systems(Update, (collect_entities, maintain_entities).chain());

        let first_id = app.world_mut().resource_mut::<EntityIdPool>().request_id().unwrap();
        let entity = app
            .world_mut()
            .spawn(GameEntity {
                unique_id: first_id,
                ref_id: 1000,
            })
            .id();
        app.update();
        assert_eq!(
            app.world().resource::<EntityLookup>().get_entity_for_id(first_id),
            Some(entity)
        );

        app.world_mut().despawn(entity);
        app.update();
        assert_eq!(app.world().resource::<EntityLookup>().get_entity_for_id(first_id), None);

        let next_id = app.world_mut().resource_mut::<EntityIdPool>().request_id().unwrap();
        assert_ne!(next_id, first_id);
    }
}
