mod agentserver;
mod cli;
mod client;
mod config;
mod login;
mod news;
mod passcode;
mod patch;
mod protocol;
mod server;

use crate::agentserver::AgentServerManager;
use crate::cli::{Cli, Commands};
use crate::config::{get_config, DbOptions, GatewayServerConfig};
use crate::login::{LoginProvider, RegistrationResult};
use crate::news::NewsCacheAsync;
use crate::patch::Patcher;
use crate::server::GatewayServer;
use clap::Parser;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use silkroad_gateway_protocol::Farm;
use sqlx::PgPool;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

const DEFAULT_FARM: &str = "Skrillax_TestBed";
const DEFAULT_HEALTHCHECK_INTERVAL: u64 = 60;
const DEFAULT_LISTEN_PORT: u16 = 15779;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let configuration = get_config()?;

    let args = Cli::parse();
    if let Some(command) = args.command {
        run_command(&command, &configuration).await?;
        return Ok(());
    }

    debug!(patch = ?configuration.patch, "patch information");

    let db_pool = create_db(&configuration.database).await?;
    let news = NewsCacheAsync::new(
        db_pool.clone(),
        Duration::from_secs(configuration.news_cache_duration.unwrap_or(120)),
    )
    .await;

    let farms = configuration
        .farms
        .clone()
        .unwrap_or_else(|| vec![DEFAULT_FARM.to_string()])
        .into_iter()
        .enumerate()
        .map(|(i, name)| Farm {
            id: (i + 1) as u8,
            name,
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

    let listen_addr = match configuration.listen_address.as_ref() {
        Some(addr) => {
            let port = configuration.listen_port.unwrap_or(DEFAULT_LISTEN_PORT);
            format!("{addr}:{port}")
                .parse()
                .expect("Listen address should be a valid ip address and port")
        },
        None => SocketAddr::new(
            Ipv4Addr::new(0, 0, 0, 0).into(),
            configuration.listen_port.unwrap_or(DEFAULT_LISTEN_PORT),
        ),
    };

    let patcher = configuration
        .patch
        .clone()
        .map(Patcher::new)
        .unwrap_or_else(Patcher::allow_all);

    let cancellation = CancellationToken::new();
    let server = GatewayServer::new(
        listen_addr,
        cancellation.clone(),
        news,
        patcher,
        LoginProvider::new(db_pool),
        agent_server_manager,
    );

    match server.run().await {
        Ok(_) => {},
        Err(e) => {
            error!(error = %e, "Could not start server")
        },
    }

    Ok(())
}

async fn create_db(configuration: &DbOptions) -> Result<PgPool> {
    configuration.create_pool().await.context("Trying to access database")
}

async fn run_command(command: &Commands, configuration: &GatewayServerConfig) -> Result<()> {
    match command {
        Commands::Register {
            username,
            password,
            passcode,
        } => {
            let db = create_db(&configuration.database).await?;
            let login_handler = LoginProvider::new(db);
            match login_handler
                .register(username, password, passcode.as_ref().map(|r| r.as_ref()))
                .await
            {
                RegistrationResult::Success => {
                    info!("Registered user {} with password {}.", username, password)
                },
                RegistrationResult::UsernameTaken => {
                    return Err(eyre!(
                        "Could not create account, because an account with that username already exists."
                    ));
                },
                _ => {
                    return Err(eyre!("Could not create account."));
                },
            }
        },
    }

    Ok(())
}
