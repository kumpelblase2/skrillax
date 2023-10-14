use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::ext::DbPool;
use crate::mall::db::{delete_expired_mall_keys, insert_user_mall_key};
use crate::mall::event::MallOpenRequestEvent;
use crate::server_plugin::ServerId;
use crate::tasks::TaskCreator;
use bevy_ecs::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use silkroad_protocol::inventory::{OpenItemMallResponse, OpenItemMallResult};
use sqlx::PgPool;
use tracing::debug;

const MALL_TOKEN_SIZE: usize = 30;

pub(crate) fn clean_tokens(db: Res<DbPool>, task_creator: Res<TaskCreator>) {
    task_creator.spawn(delete_expired_mall_keys(PgPool::clone(&db)));
}

pub(crate) fn open_mall(
    mut events: EventReader<MallOpenRequestEvent>,
    query: Query<(&Client, &Player)>,
    task_creator: Res<TaskCreator>,
    db: Res<DbPool>,
    server_id: Res<ServerId>,
) {
    for event in events.iter() {
        if let Ok((client, player)) = query.get(event.0) {
            debug!("Requesting Mall from {}", player.user.username);

            let token = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(MALL_TOKEN_SIZE)
                .map(char::from)
                .collect::<String>();

            let user_id = player.user.id as u32;
            task_creator.spawn(insert_user_mall_key(
                PgPool::clone(&db),
                user_id,
                server_id.0,
                token.clone(),
                player.character.id,
            ));

            client.send(OpenItemMallResponse(OpenItemMallResult::Success {
                jid: user_id,
                token,
            }));
        }
    }
}
