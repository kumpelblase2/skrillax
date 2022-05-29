use sqlx::PgPool;

pub(crate) struct PlayerLoader {
    pool: PgPool,
}

impl PlayerLoader {
    pub fn new(pool: PgPool) -> Self {
        PlayerLoader { pool }
    }

    // pub(crate) async fn load_player_data(&self, character: Character) -> PlayerData {}
}
