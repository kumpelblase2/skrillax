use crate::comp::net::Client;
use crate::config::GameConfig;
use crate::event::ClientDisconnectedEvent;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use bevy_time::{Time, Timer, TimerMode};
use silkroad_protocol::auth::{LogoutFinished, LogoutResponse, LogoutResult};
use std::time::Duration;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct Logout(Timer);

impl Logout {
    pub(crate) fn new(logout_duration: Duration) -> Self {
        Logout(Timer::new(logout_duration, TimerMode::Once))
    }

    pub(crate) fn from_seconds(seconds: u64) -> Self {
        Self::new(Duration::from_secs(seconds))
    }
}

pub(crate) fn handle_logout(
    query: Query<(Entity, &Client, &PlayerInput)>,
    settings: Res<GameConfig>,
    mut cmd: Commands,
) {
    for (entity, client, input) in query.iter() {
        if let Some(ref logout) = input.logout {
            client.send(LogoutResponse::new(LogoutResult::success(
                settings.logout_duration as u32,
                logout.mode,
            )));
            cmd.entity(entity)
                .try_insert(Logout::from_seconds(settings.logout_duration as u64));
        }
    }
}

pub(crate) fn tick_logout(
    mut query: Query<(Entity, &Client, &mut Logout)>,
    time: Res<Time>,
    mut writer: EventWriter<ClientDisconnectedEvent>,
) {
    let delta = time.delta();
    for (entity, client, mut logout) in query.iter_mut() {
        logout.0.tick(delta);
        if logout.0.just_finished() {
            client.send(LogoutFinished);
            writer.send(ClientDisconnectedEvent(entity));
        }
    }
}
