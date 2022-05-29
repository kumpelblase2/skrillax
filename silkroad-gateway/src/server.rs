use crate::client::Client;
use crate::login::LoginProvider;
use crate::news::NewsCacheAsync;
use crate::patch::Patcher;
use crate::AgentServerManager;
use silkroad_network::sid::StreamId;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
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
        let listener = TcpListener::bind(self.socket).await?;
        info!("Server up and accepting clients.");
        while let Some(connection) = tokio::select! {
            connected = listener.accept() => Some(connected),
            _ = self.cancellation.cancelled() => None
        } {
            if let Ok((socket, addr)) = connection {
                let id = StreamId::default();
                debug!(?addr, ?id, "Accepted client");
                let socket_cancel = self.cancellation.clone();
                let news = self.news.clone();
                let patcher = self.patcher.clone();
                let login_provider = self.login_provider.clone();
                let agent_servers = self.agent_servers.clone();
                tokio::spawn(Client::handle_client(
                    id,
                    socket,
                    socket_cancel,
                    news,
                    patcher,
                    login_provider,
                    agent_servers,
                ));
            }
        }
        Ok(())
    }
}
