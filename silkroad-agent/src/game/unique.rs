use crate::comp::monster::Monster;
use crate::comp::net::Client;
use crate::comp::GameEntity;
use crate::event::UniqueKilledEvent;
use bevy_ecs::prelude::*;
use silkroad_definitions::rarity::EntityRarityType;
use silkroad_protocol::world::GameNotification;

pub(crate) fn unique_spawned(query: Query<(&GameEntity, &Monster), Added<Monster>>, notify: Query<&Client>) {
    for (entity, _) in query
        .iter()
        .filter(|(_, monster)| monster.rarity == EntityRarityType::Unique)
    {
        notify.iter().for_each(|client| {
            client.send(GameNotification::uniquespawned(entity.ref_id));
        });
    }
}

pub(crate) fn unique_killed(mut events: EventReader<UniqueKilledEvent>, notify: Query<&Client>) {
    for kill in events.iter() {
        notify.iter().for_each(|client| {
            client.send(GameNotification::uniquekilled(kill.unique.ref_id, kill.player.clone()));
        });
    }
}
