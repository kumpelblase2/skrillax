use sqlx::{Error, PgPool};
use std::borrow::Borrow;
use std::net::SocketAddr;

pub struct ServerRegistration;

impl ServerRegistration {
    pub async fn setup<T: Borrow<PgPool>>(
        server_id: u16,
        server_name: String,
        region: String,
        listen: SocketAddr,
        rpc_port: u16,
        token: String,
        pool: T,
    ) -> Result<(), Error> {
        let port = listen.port();
        let ip = listen.ip().to_string();

        sqlx::query!(
            "INSERT INTO servers(identifier, name, region, address, port, token, rpc_port) \
            VALUES($1, $2, $3, $4, $5, $6, $7) ON CONFLICT(identifier) DO UPDATE \
            SET name = $2, region = $3, address = $4, port = $5, token = $6, rpc_port = $7",
            server_id as i16,
            server_name,
            region,
            ip,
            port as i16,
            token,
            rpc_port as i16
        )
        .execute(pool.borrow())
        .await?;
        Ok(())
    }
}
