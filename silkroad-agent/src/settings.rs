use crate::config::GameConfig;

const DEFAULT_MAX_LEVEL: u8 = 110;
const DEFAULT_CLIENT_TIMEOUT: u8 = 30;
const DEFAULT_LOGOUT_DURATION: u8 = 2;
const DEFAULT_TICKS: u32 = 128;

pub(crate) struct GameSettings {
    pub(crate) max_level: u8,
    pub(crate) client_timeout: u8,
    pub(crate) logout_duration: u8,
    pub(crate) join_notice: Option<String>,
    pub(crate) data_location: String,
    pub(crate) desired_ticks: u32,
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
        }
    }
}
