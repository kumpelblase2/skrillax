use crate::client::Client;
use crate::login::LoginProvider;
use crate::news::NewsCacheAsync;
use crate::patch::Patcher;
use crate::protocol::GatewayPacketRegistryExt;
use crate::AgentServerManager;
use silkroad_gateway_protocol::BasePacketRegistryExt;
use skrillax_server::Server;
use skrillax_stream::handshake::HandshakePacketRegistryExt;
use skrillax_stream::registry::PacketRegistry;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

pub(crate) struct GatewayServer {
    news: Arc<Mutex<NewsCacheAsync>>,
    patcher: Arc<Patcher>,
    socket: SocketAddr,
    cancellation: CancellationToken,
    login_provider: Arc<LoginProvider>,
    agent_servers: AgentServerManager,
}

impl GatewayServer {
    pub fn new(
        socket: SocketAddr,
        cancel: CancellationToken,
        news: NewsCacheAsync,
        patcher: Patcher,
        login_provider: LoginProvider,
        agent_servers: AgentServerManager,
    ) -> Self {
        GatewayServer {
            news: Arc::new(Mutex::new(news)),
            cancellation: cancel,
            socket,
            patcher: Arc::new(patcher),
            login_provider: Arc::new(login_provider),
            agent_servers,
        }
    }

    pub async fn run(self) -> Result<(), io::Error> {
        let server = Server::new(
            self.socket,
            PacketRegistry::builder()
                .register_gateway_packets()
                .register_active_handshake()
                .register_base_packets()
                .build()
                .expect("Should be able to build registry."),
        )?;
        info!("Server up and accepting clients.");
        while let Some(connection) = tokio::select! {
            connected = server.await_client() => Some(connected),
            _ = self.cancellation.cancelled() => None
        } {
            debug!(
                id = connection.id(),
                socket = ?connection.remote_address(),
                "Accepted client"
            );
            let socket_cancel = self.cancellation.child_token();
            let news = self.news.clone();
            let patcher = self.patcher.clone();
            let login_provider = self.login_provider.clone();
            let agent_servers = self.agent_servers.clone();
            tokio::spawn(Client::handle_client(
                connection,
                socket_cancel,
                news,
                patcher,
                login_provider,
                agent_servers,
            ));
        }
        Ok(())
    }
}
