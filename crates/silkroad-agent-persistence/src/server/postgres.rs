use super::{ServerRegistration, ServerWriteError};
use sqlx::PgPool;

pub(super) async fn register_server(pool: &PgPool, registration: ServerRegistration) -> Result<(), ServerWriteError> {
    let port = registration.listen_address.port();
    let ip = registration.listen_address.ip().to_string();

    sqlx::query!(
        "INSERT INTO servers(identifier, name, region, address, port, token, rpc_address, rpc_port) \
        VALUES($1, $2, $3, $4, $5, $6, $8, $7) ON CONFLICT(identifier) DO UPDATE \
        SET name = $2, region = $3, address = $4, port = $5, token = $6, rpc_address = $8, rpc_port = $7",
        registration.identifier as i16,
        registration.name,
        registration.region,
        ip,
        port as i16,
        registration.token,
        registration.rpc_port as i16,
        registration.rpc_address,
    )
    .execute(pool)
    .await?;
    Ok(())
}
