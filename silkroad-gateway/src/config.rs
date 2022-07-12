use config::ConfigError;
use lazy_static::lazy_static;
use log::LevelFilter;
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::debug;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GameServerConfig {
    pub(crate) region: String,
    pub(crate) address: String,
    pub(crate) token: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PatchConfig {
    pub(crate) remote_url: String,
    pub(crate) dir: String,
    pub(crate) accept_all: bool,
    pub(crate) expected_client_version: u32,
    pub(crate) minimum_client_version: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DbOptions {
    pub(crate) host: String,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) database: String,
    pub(crate) max_connections: Option<u32>,
}

impl DbOptions {
    pub(crate) async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        let mut options = PgPoolOptions::new();
        if let Some(max_conn) = &self.max_connections {
            options = options.max_connections(*max_conn);
        }

        debug!(username = ?self.user, host = ?self.host, database = ?self.database, "Connecting to db");

        let mut connect_options = PgConnectOptions::new()
            .username(&self.user)
            .password(&self.password)
            .host(&self.host)
            .database(&self.database);

        connect_options.log_statements(LevelFilter::Debug);

        options.connect_with(connect_options).await
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GatewayServerConfig {
    pub(crate) listen_port: u16,
    pub(crate) listen_address: Option<String>,
    pub(crate) patch: PatchConfig,
    pub(crate) servers: HashMap<String, GameServerConfig>,
    pub(crate) database: DbOptions,
    pub(crate) news_cache_duration: Option<u64>,
    pub(crate) agent_healthcheck_interval: Option<u64>,
    pub(crate) farms: Option<Vec<String>>,
}

impl GatewayServerConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        let mut config = ::config::Config::new();
        config
            .merge(config::File::with_name("configs/gateway_server"))
            .unwrap()
            .merge(config::Environment::with_prefix("SKRILLAX_GATEWAY"))
            .unwrap();
        config.try_into()
    }
}

pub(crate) fn get_config() -> &'static GatewayServerConfig {
    &CONFIG
}

lazy_static! {
    static ref CONFIG: GatewayServerConfig = GatewayServerConfig::load().expect("Could not load config.");
}
