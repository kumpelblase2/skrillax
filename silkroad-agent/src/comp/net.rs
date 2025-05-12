use bevy::prelude::*;
use derive_more::Deref;
use skrillax_server::Connection;
use skrillax_stream::stream::DynamicPacket;
use std::fmt::Debug;
use std::time::Instant;

#[derive(Component)]
pub(crate) struct LastAction(pub(crate) Instant);

#[derive(Component, Deref)]
pub(crate) struct Client(pub(crate) Connection);

impl Client {
    pub fn send<T: Into<DynamicPacket> + Debug>(&self, packet: T) {
        // We specifically ignore the error here because we'll handle the client being disconnected
        // at the end of the game tick. This means we might do some unnecessary things, but that's ok
        // for now. The upside is that this means there's a single point where we handle such errors.
        let _ = self.0.send(packet);
    }
}
