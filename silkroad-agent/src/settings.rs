use crate::config::GameConfig;

const DEFAULT_MAX_LEVEL: u8 = 110;
const DEFAULT_CLIENT_TIMEOUT: u8 = 30;
const DEFAULT_LOGOUT_DURATION: u8 = 2;
const DEFAULT_TICKS: u32 = 128;
const DEFAULT_DELETION_TIME: u32 = 10080;
// 7 days in minutes
const DEFAULT_SPAWN_RADIUS: f32 = 500.0;
const DEFAULT_SPAWN_AMOUNT: usize = 10;

pub(crate) struct SpawnSettings {
    pub(crate) radius: f32,
    pub(crate) amount: usize,
}

pub(crate) struct GameSettings {
    pub(crate) max_level: u8,
    pub(crate) client_timeout: u8,
    pub(crate) logout_duration: u8,
    pub(crate) join_notice: Option<String>,
    pub(crate) data_location: String,
    pub(crate) desired_ticks: u32,
    pub(crate) deletion_time: u32,
    pub(crate) spawn_settings: SpawnSettings,
}

impl From<GameConfig> for GameSettings {
    fn from(config: GameConfig) -> GameSettings {
        GameSettings {
            max_level: config.max_level.unwrap_or(DEFAULT_MAX_LEVEL),
            client_timeout: config.client_timeout.unwrap_or(DEFAULT_CLIENT_TIMEOUT),
            logout_duration: config.logout_duration.unwrap_or(DEFAULT_LOGOUT_DURATION),
            join_notice: config.join_notice,
            data_location: config.data_location,
            desired_ticks: config.desired_ticks.unwrap_or(DEFAULT_TICKS),
            deletion_time: config.deletion_time.unwrap_or(DEFAULT_DELETION_TIME),
            spawn_settings: config
                .spawner
                .map(|spawn| SpawnSettings {
                    radius: spawn.radius.unwrap_or(DEFAULT_SPAWN_RADIUS),
                    amount: spawn.amount.unwrap_or(DEFAULT_SPAWN_AMOUNT),
                })
                .unwrap_or_else(|| SpawnSettings {
                    radius: DEFAULT_SPAWN_RADIUS,
                    amount: DEFAULT_SPAWN_AMOUNT,
                }),
        }
    }
}
