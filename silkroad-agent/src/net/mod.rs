use crate::event::{ClientConnectedEvent, ClientDisconnectedEvent};
use crate::ext::ServerResource;
use crate::net::net::{accept, connected, disconnected};
use bevy::prelude::*;
use silkroad_protocol::auth::AuthPacketRegistryExt;
use silkroad_protocol::character::CharacterPacketRegistryExt;
use silkroad_protocol::chat::ChatPacketRegistryExt;
use silkroad_protocol::combat::CombatPacketRegistryExt;
use silkroad_protocol::community::CommunityPacketRegistryExt;
use silkroad_protocol::gm::GmPacketRegistryExt;
use silkroad_protocol::inventory::InventoryPacketRegistryExt;
use silkroad_protocol::movement::MovementPacketRegistryExt;
use silkroad_protocol::skill::SkillPacketRegistryExt;
use silkroad_protocol::spawn::SpawnPacketRegistryExt;
use silkroad_protocol::wire_entity::WireEntityCatalog;
use silkroad_protocol::world::WorldPacketRegistryExt;
use silkroad_protocol::BasePacketRegistryExt;
use skrillax_server::Server;
use skrillax_stream::handshake::HandshakePacketRegistryExt;
use skrillax_stream::registry::PacketRegistry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod net;

pub struct NetworkPlugin {
    server: SocketAddr,
    runtime: Arc<Runtime>,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Need to run this inside a `block_on` to ensure we're inside tokio and can
        // `spawn()` more tasks.
        let server = self.runtime.block_on(async {
            Server::new_with_context_initializer(
                self.server,
                PacketRegistry::builder()
                    .register_base_packets()
                    .register_active_handshake()
                    .register_auth_packets()
                    .register_character_packets()
                    .register_chat_packets()
                    .register_combat_packets()
                    .register_community_packets()
                    .register_gm_packets()
                    .register_inventory_packets()
                    .register_movement_packets()
                    .register_skill_packets()
                    .register_spawn_packets()
                    .register_world_packets()
                    .build()
                    .expect("Should be able to create registry."),
                |context| {
                    WireEntityCatalog::install(context);
                },
            )
            .expect("Should be able to create the server")
            .into()
        });

        app.insert_resource::<ServerResource>(server)
            .add_systems(First, (accept, connected))
            .add_systems(Last, disconnected)
            .add_message::<ClientDisconnectedEvent>()
            .add_message::<ClientConnectedEvent>();
    }
}

impl NetworkPlugin {
    pub fn new(server: SocketAddr, runtime: Arc<Runtime>) -> Self {
        Self { server, runtime }
    }
}
