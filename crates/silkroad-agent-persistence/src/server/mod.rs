mod postgres;

use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct ServerPersistence {
    pool: PgPool,
}

impl ServerPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, registration: ServerRegistration) -> Result<(), ServerWriteError> {
        postgres::register_server(&self.pool, registration).await
    }
}

#[derive(Clone, Debug)]
pub struct ServerRegistration {
    pub identifier: u16,
    pub name: String,
    pub region: String,
    pub listen_address: SocketAddr,
    pub rpc_address: String,
    pub rpc_port: u16,
    pub token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerWriteError {
    #[error("server database query failed: {0}")]
    Database(#[from] sqlx::Error),
}
