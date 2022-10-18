use sqlx::PgPool;
use std::net::SocketAddr;

pub(crate) async fn register_server(
    pool: PgPool,
    server_name: String,
    region: String,
    listen: SocketAddr,
    rpc_port: u16,
    token: String,
) {
    let port = listen.port();
    let ip = listen.ip().to_string();

    sqlx::query!(
        "INSERT INTO servers(name, region, address, port, rpc_port, token) VALUES($1, $2, $3, $4, $6, $5) ON CONFLICT(name) DO UPDATE SET region = $2, address = $3, port = $4, token = $5, rpc_port = $6",
        server_name,
        region,
        ip,
        port as i16,
        token,
        rpc_port as i16
    )
            .execute(&pool)
            .await
            .expect("Should be able to insert");
}
