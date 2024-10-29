use crate::protocol::AgentClientProtocol;
use bevy_ecs::prelude::*;
use derive_more::Deref;
use skrillax_protocol::__internal::AsPacket;
use skrillax_server::Connection;
use std::time::Instant;

#[derive(Component)]
pub(crate) struct LastAction(pub(crate) Instant);

#[derive(Component, Deref)]
pub(crate) struct Client(pub(crate) Connection<AgentClientProtocol>);

impl Client {
    pub fn send<T: AsPacket + Send + 'static>(&self, packet: T) {
        // We specifically ignore the error here because we'll handle the client being disconnected
        // at the end of the game tick. This means we might do some unnecessary things, but that's ok
        // for now. The upside is that this means there's a single point where we handle such errors.
        let _ = self.0.send(packet);
    }
}
