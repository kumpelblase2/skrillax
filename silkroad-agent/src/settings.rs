use crate::config::GameConfig;

pub(crate) struct GameSettings {
    pub(crate) max_level: u8,
    pub(crate) client_timeout: u8,
    pub(crate) logout_duration: u8,
    pub(crate) join_notice: Option<String>,
}

impl From<GameConfig> for GameSettings {
    fn from(config: GameConfig) -> GameSettings {
        GameSettings {
            max_level: config.max_level.unwrap_or(110),
            client_timeout: config.client_timeout.unwrap_or(30),
            logout_duration: config.logout_duration.unwrap_or(2),
            join_notice: config.join_notice,
        }
    }
}
