use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::Health;
use crate::config::GameConfig;
use crate::db::character::CharacterData;
use crate::event::ClientDisconnectedEvent;
use crate::ext::DbPool;
use crate::persist::Persistable;
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_time::Time;
use tracing::warn;

pub(crate) fn attach_persistence(mut cmd: Commands, query: Query<Entity, Added<Player>>, config: Res<GameConfig>) {
    for player_entity in query.iter() {
        cmd.entity(player_entity)
            .insert(Persistable::from_seconds(config.auto_save_interval));
    }
}

pub(crate) fn run_persistence(
    runtime: Res<TaskCreator>,
    delta: Res<Time>,
    db_pool: Res<DbPool>,
    mut query: Query<(&mut Persistable, &Health, &Player, &Position)>,
) {
    let duration = delta.delta();
    let mut updates: Vec<(Health, Player, Position)> = Vec::new();
    for (mut persistable, health, player, position) in query.iter_mut() {
        if persistable.should_persist(duration) {
            updates.push((*health, player.clone(), *position));
        }
    }

    let pool = db_pool.clone();
    runtime.spawn(async move {
        for (health, player, position) in updates.into_iter() {
            CharacterData::update_character_info(player, health, position, &pool).await;
        }
    });
}

pub(crate) fn run_exit_persistence(
    runtime: Res<TaskCreator>,
    db_pool: Res<DbPool>,
    query: Query<(&Health, &Player, &Position)>,
    mut disconnect_events: EventReader<ClientDisconnectedEvent>,
) {
    for event in disconnect_events.iter() {
        match query.get(event.0) {
            Ok((health, player, position)) => {
                let health = *health;
                let player = player.clone();
                let position = *position;
                let pool = db_pool.clone();
                runtime.spawn(async move {
                    CharacterData::update_character_info(player, health, position, pool).await;
                });
            },
            Err(QueryEntityError::NoSuchEntity(_)) => {
                warn!("Couldn't run persistence for entity on exit as it was already removed.");
            },
            _ => {},
        }
    }
}
