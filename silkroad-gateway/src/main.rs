mod agentserver;
mod client;
mod config;
mod login;
mod news;
mod patch;
mod server;

use crate::agentserver::{AgentServer, AgentServerManager};
use crate::config::get_config;
use crate::login::LoginProvider;
use crate::news::NewsCacheAsync;
use crate::patch::Patcher;
use crate::server::GatewayServer;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let configuration = get_config();

    debug!(patch = ?configuration.patch, "patch information");

    let db_pool = configuration.database.create_pool().await.unwrap();
    let news = NewsCacheAsync::new(
        db_pool.clone(),
        Duration::from_secs(configuration.news_cache_duration.unwrap_or(120)),
    )
    .await;

    let agent_server_manager = AgentServerManager::new(Duration::from_secs(
        configuration.agent_healthcheck_interval.unwrap_or(60),
    ));

    let mut id = 1;
    for (name, server) in configuration.servers.iter() {
        agent_server_manager
            .add_server(AgentServer::new(
                id,
                name.clone(),
                server.address.clone(),
                server.region.clone().into(),
            ))
            .await;
        id += 1;
    }

    let listen_addr = SocketAddr::from_str(&format!(
        "{}:{}",
        configuration
            .listen_address
            .as_ref()
            .unwrap_or(&String::from("0.0.0.0")),
        configuration.listen_port
    ))
    .expect("Should be a valid listen address.");
    let cancellation = CancellationToken::new();
    let server = GatewayServer::new(
        listen_addr,
        cancellation.clone(),
        news,
        Patcher::new(configuration.patch.clone()),
        LoginProvider::new(db_pool),
        agent_server_manager,
    );

    match server.run().await {
        Ok(_) => {}
        Err(e) => {
            error!(error = %e, "Could not start server")
        }
    }
}
