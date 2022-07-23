use bevy_ecs::prelude::*;

pub(crate) struct ClientConnectedEvent(pub Entity);

pub(crate) struct ClientDisconnectedEvent(pub Entity);

pub(crate) struct PlayerLevelUp(pub Entity, pub u8);

pub(crate) struct LoadingFinishedEvent(pub Entity);
