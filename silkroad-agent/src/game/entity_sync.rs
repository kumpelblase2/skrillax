use crate::comp::player::Player;
use crate::comp::sync::{MovementUpdate, Synchronize};
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity};
use bevy_ecs::prelude::*;
use silkroad_protocol::world::{MovementDestination, MovementSource, PlayerMovementResponse};
use silkroad_protocol::ServerPacket;
use tracing::debug;

pub(crate) fn sync_changes(
    query: Query<(&Client, &Visibility), With<Player>>,
    others: Query<(&GameEntity, &Synchronize)>,
) {
    for (client, visibility) in query.iter() {
        let client: &Client = client;
        let visibility: &Visibility = visibility;

        for (entity, synchronize) in visibility
            .entities_in_radius
            .iter()
            .map(|entity| others.get(*entity))
            .filter_map(|res| res.ok())
        {
            let entity: &GameEntity = entity;
            let synchronize: &Synchronize = synchronize;
            if let Some(movement) = &synchronize.movement {
                update_movement_for(client, entity, movement);
            }
        }
    }
}

fn update_movement_for(client: &Client, entity: &GameEntity, movement: &MovementUpdate) {
    match movement {
        MovementUpdate::StartMove(current, target) => {
            debug!("Sending start movement to client for entity: {}", entity.unique_id);
            client.send(ServerPacket::PlayerMovementResponse(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::location(target.0.id(), target.1.x as u16, target.1.y as u16, target.1.z as u16),
                Some(MovementSource::new(
                    current.0.id(),
                    (current.1.x * 10.) as u16,
                    current.1.y * 10.,
                    (current.1.z * 10.) as u16,
                )),
            )));
        },
        MovementUpdate::StopMove(current) => {
            debug!("Sending stop movement to client for entity: {}", entity.unique_id);
            client.send(ServerPacket::PlayerMovementResponse(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::location(
                    current.0.id(),
                    current.1.x as u16,
                    current.1.y as u16,
                    current.1.z as u16,
                ),
                None,
            )));
        },
        MovementUpdate::Turn(heading) => {
            client.send(ServerPacket::PlayerMovementResponse(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::direction(false, heading.clone().into()),
                None,
            )));
        },
    }
}

pub(crate) fn update_client(query: Query<(&Client, &GameEntity, &Synchronize)>) {
    for (client, entity, sync) in query.iter() {
        let client: &Client = client;
        let sync: &Synchronize = sync;

        if let Some(movement) = &sync.movement {
            update_movement_for(client, entity, movement);
        }
    }
}

pub(crate) fn clean_sync(mut query: Query<&mut Synchronize>) {
    for mut sync in query.iter_mut() {
        sync.clear();
    }
}
