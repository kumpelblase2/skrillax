use bevy::prelude::*;
use config::{ConfigError, FileFormat};
use log::LevelFilter;
use once_cell::sync::Lazy;
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::ops::RangeInclusive;
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

        let connect_options = PgConnectOptions::new()
            .username(&self.user)
            .password(&self.password)
            .host(&self.host)
            .database(&self.database)
            .log_statements(LevelFilter::Debug);

        options.connect_with(connect_options).await
    }
}

#[derive(Deserialize, Default, Clone, Resource)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GameConfig {
    pub(crate) max_level: u8,
    pub(crate) client_timeout: u8,
    pub(crate) logout_duration: u8,
    pub(crate) join_notice: Option<String>,
    pub(crate) data_location: String,
    pub(crate) desired_ticks: u32,
    pub(crate) deletion_time: u32,
    pub(crate) spawner: SpawnOptions,
    pub(crate) max_follow_distance: f32,
    pub(crate) masteries: MasteryConfig,
    pub(crate) persist_interval: u64,
    pub(crate) drop: DropConfig,
}

#[derive(Deserialize, Default, Clone)]
pub(crate) struct SpawnOptions {
    pub(crate) radius: f32,
    pub(crate) amount: usize,
    pub(crate) unique: UniqueOptions,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct UniqueOptions {
    pub(crate) tiger_woman: UniqueSpawnOptions,
    pub(crate) uruchi: UniqueSpawnOptions,
    pub(crate) isyutaru: UniqueSpawnOptions,
    pub(crate) bonelord: UniqueSpawnOptions,
}

#[derive(Deserialize, Default, Clone)]
pub(crate) struct UniqueSpawnOptions {
    pub(crate) min: usize,
    pub(crate) max: usize,
}

impl UniqueSpawnOptions {
    pub(crate) fn spawn_range(&self) -> RangeInclusive<usize> {
        self.min..=self.max
    }
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct MasteryConfig {
    pub(crate) european_per_level: u16,
    pub(crate) chinese_per_level: u16,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DropConfig {
    pub(crate) gold: f32,
    pub(crate) experience: f32,
    pub(crate) sp_experience: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GameServerConfig {
    pub(crate) listen_port: u16,
    pub(crate) listen_address: String,
    pub(crate) external_address: Option<String>,
    pub(crate) server_id: u16,
    pub(crate) rpc_address: String,
    pub(crate) rpc_port: u16,
    pub(crate) max_player_count: u16,
    pub(crate) database: DbOptions,
    pub(crate) game: GameConfig,
    pub(crate) region: String,
    pub(crate) name: String,
}

static DEFAULT_CONFIG: &str = include_str!("../conf/default.toml");

impl GameServerConfig {
    pub(crate) fn load() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(config::File::with_name("configs/agent_server"))
            .add_source(config::Environment::with_prefix("SKRILLAX_AGENT").separator("_"))
            .build()
            .and_then(|c| c.try_deserialize())
    }
}

pub(crate) fn get_config() -> &'static GameServerConfig {
    &CONFIG
}

static CONFIG: Lazy<GameServerConfig> = Lazy::new(|| GameServerConfig::load().expect("Should be able to load config"));
