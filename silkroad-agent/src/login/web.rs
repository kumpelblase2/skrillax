use crate::db::user::ServerUser;
use crate::population::ReservationError;
use crate::{CapacityController, LoginQueue};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Extension, Json, Router, Server};
use silkroad_rpc::{ReserveRequest, ReserveResponse, ServerStatusReport};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::error;

#[derive(Clone)]
struct Capacity(CapacityController);

#[derive(Clone)]
struct Settings(u16, String);

async fn handle_capacity(Extension(capacity): Extension<Capacity>) -> Json<ServerStatusReport> {
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
    headers: HeaderMap,
) -> Json<ReserveResponse> {
    let Some(passed_token) = headers.get("TOKEN") else {
        return Json(ReserveResponse::Error("Missing auth token.".to_string()));
    };
    let Ok(passed_token) = passed_token.to_str() else {
        return Json(ReserveResponse::Error("Invalid auth token.".to_string()));
    };

    if passed_token != settings.1 {
        return Json(ReserveResponse::Error("Invalid auth token.".to_string()));
    }

    let user = match ServerUser::fetch(reservation.user_id, settings.0, pool).await {
        Ok(Some(user)) => user,
        Ok(None) => return Json(ReserveResponse::NotFound),
        Err(e) => {
            error!(token = passed_token, "Could not fetch user from db: {}", e);
            return Json(ReserveResponse::NotFound);
        },
    };
    match login_queue.reserve_spot(user) {
        Ok((id, duration)) => Json(ReserveResponse::Success {
            token: id,
            alive: duration.as_secs(),
        }),
        Err(ReservationError::AllTokensTaken) | Err(ReservationError::NoSpotsAvailable) => Json(ReserveResponse::Full),
        Err(ReservationError::NoSuchToken) => Json(ReserveResponse::NotFound), // This should never happen.
        Err(ReservationError::AlreadyHasReservation) => Json(ReserveResponse::Duplicate),
    }
}

pub(crate) struct WebServer;

impl WebServer {
    pub async fn run(
        server_id: u16,
        pool: PgPool,
        login_queue: LoginQueue,
        capacity: CapacityController,
        token: String,
        port: u16,
    ) {
        let router = Router::new()
            .route("/status", get(handle_capacity))
            .route("/request", post(handle_spot_request))
            .layer(Extension(Capacity(capacity)))
            .layer(Extension(Settings(server_id, token)))
            .layer(Extension(login_queue))
            .layer(Extension(pool));

        // TODO: this should be configurable on where it listens on
        let socket_addr = SocketAddr::from(([0, 0, 0, 0], port));

        Server::bind(&socket_addr)
            .serve(router.into_make_service())
            .await
            .expect("Should be able to bind webserver socket");
    }
}
