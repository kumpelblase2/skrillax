mod agentserver;
mod client;
mod config;
mod login;
mod news;
mod patch;
mod server;

use crate::agentserver::AgentServerManager;
use crate::config::get_config;
use crate::login::LoginProvider;
use crate::news::NewsCacheAsync;
use crate::patch::Patcher;
use crate::server::GatewayServer;
use silkroad_protocol::login::Farm;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

const DEFAULT_FARM: &str = "Skrillax_TestBed";
const DEFAULT_HEALTHCHECK_INTERVAL: u64 = 60;

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

    let farms = configuration
        .farms
        .clone()
        .unwrap_or(vec![DEFAULT_FARM.to_string()])
        .into_iter()
        .enumerate()
        .map(|(i, name)| Farm {
            id: (i + 1) as u8,
            name: name.clone(),
        })
        .collect();

    let agent_server_manager = AgentServerManager::new(
        Duration::from_secs(
            configuration
                .agent_healthcheck_interval
                .unwrap_or(DEFAULT_HEALTHCHECK_INTERVAL),
        ),
        farms,
        db_pool.clone(),
    );

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
        Ok(_) => {},
        Err(e) => {
            error!(error = %e, "Could not start server")
        },
    }
}
