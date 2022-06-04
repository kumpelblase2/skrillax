use bevy_ecs::prelude::*;

pub(crate) struct ClientConnectedEvent(pub Entity);

pub(crate) struct ClientDisconnectedEvent(pub Entity);
