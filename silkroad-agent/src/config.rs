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
    pub(crate) loot: LootConfig,
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
    pub(crate) european: Vec<ConfiguredMastery>,
    pub(crate) chinese: Vec<ConfiguredMastery>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConfiguredMastery {
    pub(crate) ref_id: u16,
    pub(crate) secondary_id: Option<u8>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DropConfig {
    pub(crate) experience: f32,
    pub(crate) sp_experience: f32,
}

/// Operational settings for the custom loot table system.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct LootConfig {
    /// Directory containing loot definition `.ron` files. Relative to the
    /// server process working directory.
    pub(crate) directory: String,
    /// Global multiplier on expected successful pool selections.
    pub(crate) rate: f64,
}

impl Default for LootConfig {
    fn default() -> Self {
        Self {
            directory: String::from("configs/loot"),
            rate: 1.0,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Defaults {
        game: GameDefaults,
    }

    #[derive(Deserialize)]
    struct GameDefaults {
        masteries: MasteryConfig,
    }

    #[test]
    fn shipped_mastery_configuration_preserves_current_race_defaults() {
        let defaults: Defaults = config::Config::builder()
            .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        let configured = |mastery: &ConfiguredMastery| (mastery.ref_id, mastery.secondary_id);

        assert_eq!(
            defaults
                .game
                .masteries
                .chinese
                .iter()
                .map(configured)
                .collect::<Vec<_>>(),
            vec![
                (257, None),
                (258, None),
                (259, None),
                (277, Some(0)),
                (277, Some(1)),
                (277, Some(2)),
                (267, None),
            ]
        );
        assert_eq!(
            defaults
                .game
                .masteries
                .european
                .iter()
                .map(configured)
                .collect::<Vec<_>>(),
            vec![
                (513, None),
                (514, None),
                (515, None),
                (516, None),
                (517, None),
                (518, None),
            ]
        );
    }
}
