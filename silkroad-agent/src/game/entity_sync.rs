use crate::comp::player::Player;
use crate::comp::sync::{MovementUpdate, SkillUse, Synchronize};
use crate::comp::visibility::Visibility;
use crate::comp::{Client, GameEntity};
use bevy_ecs::prelude::*;
use silkroad_protocol::combat::{ActionType, PerformActionUpdate};
use silkroad_protocol::world::{
    EntityUpdateState, MovementDestination, MovementSource, PlayerMovementResponse, UpdatedState,
};
use tracing::debug;

pub(crate) fn sync_changes_others(
    query: Query<(&Client, &Visibility), With<Player>>,
    others: Query<(&GameEntity, &Synchronize)>,
) {
    for (client, visibility) in query.iter() {
        for (entity, synchronize) in visibility
            .entities_in_radius
            .iter()
            .map(|reference| others.get(reference.0))
            .filter_map(|res| res.ok())
        {
            if let Some(movement) = &synchronize.movement {
                update_movement_for(client, entity, movement);
            }

            for state in synchronize.state.iter() {
                update_state(client, entity, *state);
            }

            if let Some(skill_use) = &synchronize.skill {
                use_skill_by(client, entity, skill_use);
            }
        }
    }
}

fn use_skill_by(client: &Client, entity: &GameEntity, skill: &SkillUse) {
    client.send(PerformActionUpdate::success(
        skill.used.ref_id,
        entity.ref_id,
        0,
        ActionType::None,
        None,
    ));
}

fn update_movement_for(client: &Client, entity: &GameEntity, movement: &MovementUpdate) {
    match movement {
        MovementUpdate::StartMove(current, target) => {
            client.send(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::location(target.0.id(), target.1.x as u16, target.1.y as u16, target.1.z as u16),
                Some(MovementSource::new(
                    current.0.id(),
                    (current.1.x * 10.) as u16,
                    current.1.y * 10.,
                    (current.1.z * 10.) as u16,
                )),
            ));
        },
        MovementUpdate::StartMoveTowards(current, direction) => {
            let angle: u16 = (*direction).into();
            debug!("Starting Movement: {}({})", direction.0, angle);
            client.send(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::direction(true, (*direction).into()),
                Some(MovementSource::new(
                    current.0.id(),
                    (current.1.x * 10.) as u16,
                    current.1.y * 10.,
                    (current.1.z * 10.) as u16,
                )),
            ));
        },
        MovementUpdate::StopMove(current, _heading) => {
            client.send(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::location(
                    current.0.id(),
                    current.1.x as u16,
                    current.1.y as u16,
                    current.1.z as u16,
                ),
                None,
            ));
        },
        MovementUpdate::Turn(heading) => {
            client.send(PlayerMovementResponse::new(
                entity.unique_id,
                MovementDestination::direction(false, (*heading).into()),
                None,
            ));
        },
    }
}

pub(crate) fn update_client(query: Query<(&Client, &GameEntity, &Synchronize)>) {
    for (client, entity, sync) in query.iter() {
        if let Some(movement) = &sync.movement {
            update_movement_for(client, entity, movement);
        }

        for state in sync.state.iter() {
            update_state(client, entity, *state);
        }

        if !sync.damage.is_empty() {
            // ...
        }
    }
}

pub(crate) fn update_state(client: &Client, entity: &GameEntity, state: UpdatedState) {
    client.send(EntityUpdateState {
        unique_id: entity.unique_id,
        update: state,
    });
}

pub(crate) fn clean_sync(mut query: Query<&mut Synchronize>) {
    for mut sync in query.iter_mut() {
        sync.clear();
    }
}
