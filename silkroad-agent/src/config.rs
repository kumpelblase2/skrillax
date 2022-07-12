use config::ConfigError;
use lazy_static::lazy_static;
use log::LevelFilter;
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use tracing::debug;

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

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GameConfig {
    pub(crate) max_level: Option<u8>,
    pub(crate) client_timeout: Option<u8>,
    pub(crate) logout_duration: Option<u8>,
    pub(crate) join_notice: Option<String>,
    pub(crate) data_location: String,
    pub(crate) desired_ticks: Option<u32>,
    pub(crate) deletion_time: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GameServerConfig {
    pub(crate) listen_port: u16,
    pub(crate) listen_address: Option<String>,
    pub(crate) external_address: Option<String>,
    pub(crate) server_id: u16,
    pub(crate) rpc_port: Option<u16>,
    pub(crate) max_player_count: u16,
    pub(crate) database: DbOptions,
    pub(crate) game: GameConfig,
}

impl GameServerConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        let mut config = ::config::Config::new();
        config
            .merge(config::File::with_name("configs/agent_server"))
            .unwrap()
            .merge(config::Environment::with_prefix("SKRILLAX_AGENT"))
            .unwrap();
        config.try_into()
    }
}

pub(crate) fn get_config() -> &'static GameServerConfig {
    &CONFIG
}

lazy_static! {
    static ref CONFIG: GameServerConfig = GameServerConfig::load().expect("Could not load config.");
}
