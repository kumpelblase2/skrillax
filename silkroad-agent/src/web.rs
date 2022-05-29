use crate::db::user::fetch_server_user;
use crate::{CapacityController, LoginQueue};
use salvo::prelude::{FlowCtrl, TcpListener};
use salvo::{Depot, Handler, Request, Response, Router, Server};
use silkroad_rpc::{ReserveRequest, ReserveResponse, ServerStatusReport};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;

pub(crate) struct HealthStatusHandler {
    capacity: Arc<CapacityController>,
}

#[salvo::async_trait]
impl Handler for HealthStatusHandler {
    async fn handle(&self, _: &mut Request, _: &mut Depot, res: &mut Response, _: &mut FlowCtrl) {
        let status = ServerStatusReport {
            healthy: true,
            population: self.capacity.usage().into(),
        };
        res.render_json(&status);
    }
}

pub(crate) struct SpotReserveHandler {
    server_id: u16,
    pool: PgPool,
    login_queue: LoginQueue,
    external_address: SocketAddr,
}

#[salvo::async_trait]
impl Handler for SpotReserveHandler {
    async fn handle(&self, req: &mut Request, _: &mut Depot, res: &mut Response, _: &mut FlowCtrl) {
        let reservation = req.read_from_json::<ReserveRequest>().await.unwrap();
        let user = fetch_server_user(&self.pool, reservation.user_id, self.server_id)
            .await
            .unwrap()
            .unwrap();
        match self.login_queue.reserve_spot(user) {
            Ok((id, duration)) => {
                res.render_json(&ReserveResponse::Success {
                    token: id,
                    ip: self.external_address.ip().to_string(),
                    port: self.external_address.port(),
                    alive: duration.as_secs(),
                });
            }
            Err(e) => {
                res.render_json(&ReserveResponse::Error(format!(
                    "No more spots available. {:?}",
                    e
                )));
            }
        }
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
        let status_router = Router::with_path("/status").get(HealthStatusHandler {
            capacity: self.capacity,
        });
        let request_router = Router::with_path("/request").post(SpotReserveHandler {
            server_id: self.server_id,
            pool: self.pool,
            login_queue: self.login_queue,
            external_address: self.external_address,
        });
        let router = Router::new().push(status_router).push(request_router);

        Server::new(TcpListener::bind(&format!("localhost:{}", self.port)))
            .serve(router)
            .await;
    }
}
