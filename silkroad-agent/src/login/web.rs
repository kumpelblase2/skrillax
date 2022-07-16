use crate::db::user::fetch_server_user;
use crate::{CapacityController, LoginQueue};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router, Server};
use silkroad_rpc::{ReserveRequest, ReserveResponse, ServerStatusReport};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct Capacity(Arc<CapacityController>);

#[derive(Clone)]
struct Settings(u16, SocketAddr);

async fn handle_capacity(Extension(capacity): Extension<Capacity>) -> impl IntoResponse {
    let status = ServerStatusReport {
        healthy: true,
        population: capacity.0.usage().into(),
    };
    Json(status)
}

async fn handle_spot_request(
    Extension(settings): Extension<Settings>,
    Extension(pool): Extension<PgPool>,
    Extension(login_queue): Extension<LoginQueue>,
    Json(reservation): Json<ReserveRequest>,
) -> impl IntoResponse {
    let user = fetch_server_user(&pool, reservation.user_id, settings.0)
        .await
        .unwrap()
        .unwrap();
    match login_queue.reserve_spot(user) {
        Ok((id, duration)) => Json(ReserveResponse::Success {
            token: id,
            ip: settings.1.ip().to_string(),
            port: settings.1.port(),
            alive: duration.as_secs(),
        }),
        Err(e) => Json(ReserveResponse::Error(format!("No more spots available. {:?}", e))),
    }
}

pub(crate) struct WebServer {
    server_id: u16,
    pool: PgPool,
    login_queue: LoginQueue,
    capacity: Arc<CapacityController>,
    external_address: SocketAddr,
    port: u16,
}

impl WebServer {
    pub(crate) fn new(
        server_id: u16,
        pool: PgPool,
        login_queue: LoginQueue,
        capacity_controller: Arc<CapacityController>,
        external_address: SocketAddr,
        port: u16,
    ) -> Self {
        WebServer {
            server_id,
            pool,
            login_queue,
            capacity: capacity_controller,
            external_address,
            port,
        }
    }

    pub async fn run(self) {
        let router = Router::new()
            .route("/status", get(handle_capacity))
            .route("/request", post(handle_spot_request))
            .layer(Extension(Capacity(self.capacity)))
            .layer(Extension(Settings(self.server_id, self.external_address)))
            .layer(Extension(self.login_queue))
            .layer(Extension(self.pool));

        // TODO: this should be configurable on where it listens on
        let socket_addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        Server::bind(&socket_addr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    }
}
