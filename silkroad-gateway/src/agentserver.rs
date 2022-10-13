use reqwest::Client;
use silkroad_protocol::login::{Farm, Shard};
use silkroad_rpc::{ReserveRequest, ReserveResponse, ServerPopulation, ServerStatusReport};
use sqlx::{PgPool, Row};
use std::fmt::Display;
use std::mem;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

#[derive(Copy, Clone)]
pub(crate) enum ServerRegion {
    US,
    EU,
    KR,
}

impl<T> From<T> for ServerRegion
where
    T: AsRef<str> + Display,
{
    fn from(value: T) -> Self {
        match value.as_ref() {
            "EU" => ServerRegion::EU,
            "US" => ServerRegion::US,
            "KR" => ServerRegion::KR,
            _ => panic!("No such region {}", value),
        }
    }
}

impl From<ServerRegion> for char {
    #[allow(clippy::let_and_return)]
    fn from(region: ServerRegion) -> Self {
        let ch = match region {
            ServerRegion::US => 0x32u8,
            ServerRegion::EU => 0x31u8,
            ServerRegion::KR => 0x30u8,
        } as char;
        ch // need this separate, otherwise the cast doesn't work
    }
}

#[derive(Copy, Clone)]
pub(crate) enum ServerStatus {
    Online,
    Offline,
}

impl From<ServerStatus> for bool {
    fn from(status: ServerStatus) -> Self {
        match status {
            ServerStatus::Online => true,
            ServerStatus::Offline => false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct AgentServer {
    pub(crate) id: u16,
    pub(crate) name: String,
    pub(crate) address: SocketAddr,
    pub(crate) rpc_address: SocketAddr,
    pub(crate) region: ServerRegion,
    pub(crate) status: ServerStatus,
    pub(crate) token: String,
    pub(crate) population: ServerPopulation,
}

impl From<AgentServer> for Shard {
    fn from(server: AgentServer) -> Self {
        let mut name = server.name;
        name.insert(0, server.region.into());

        Shard {
            id: server.id,
            name,
            status: server.population.into(),
            is_online: server.status.into(),
        }
    }
}

impl AgentServer {
    pub(crate) fn new(
        id: u16,
        name: String,
        address: SocketAddr,
        rpc: SocketAddr,
        region: ServerRegion,
        token: String,
    ) -> Self {
        AgentServer {
            id,
            name,
            address,
            rpc_address: rpc,
            region,
            token,
            status: ServerStatus::Offline,
            population: ServerPopulation::Easy,
        }
    }

    pub(crate) fn update(&mut self, report: ServerStatusReport) {
        self.status = if report.healthy {
            ServerStatus::Online
        } else {
            ServerStatus::Offline
        };
        self.population = report.population;
    }
}

#[derive(Clone)]
pub(crate) struct AgentServerManager {
    farms: Vec<Farm>,
    servers: Arc<RwLock<Vec<AgentServer>>>,
}

async fn fetch_servers(pool: PgPool) -> Vec<AgentServer> {
    let servers = match sqlx::query("SELECT id, name, region, address, port, rpc_port, token FROM servers")
        .fetch_all(&pool)
        .await
    {
        Ok(servers) => servers,
        Err(ref e) => {
            error!(error = %e, "Could not load servers from database.");
            Vec::default()
        },
    };
    servers
        .into_iter()
        .map(|row| {
            let id: i32 = row.get(0);
            let name: String = row.get(1);
            let region: String = row.get(2);
            let address: String = row.get(3);
            let port: i16 = row.get(4);
            let rpc_port: i16 = row.get(5);
            let token: String = row.get(6);

            let ip = address
                .parse()
                .expect("Address in database should be a valid ip address");
            AgentServer::new(
                id as u16,
                name,
                SocketAddr::new(ip, port as u16),
                SocketAddr::new(ip, rpc_port as u16),
                ServerRegion::from(region),
                token,
            )
        })
        .collect()
}

impl AgentServerManager {
    pub(crate) fn new(poll_interval: Duration, farms: Vec<Farm>, db: PgPool) -> Self {
        let servers: Arc<RwLock<Vec<AgentServer>>> = Arc::new(RwLock::new(Vec::new()));
        let server_copy = servers.clone();

        tokio::spawn(async move {
            loop {
                let mut servers = fetch_servers(db.clone()).await;
                for server in servers.iter_mut() {
                    let status = match Self::request_server_status(server).await {
                        Ok(resp) => resp,
                        Err(e) if e.is_request() || e.is_timeout() || e.is_connect() => {
                            debug!(server = ?server.name, error = %e, "Agent server was not accessible.");
                            ServerStatusReport {
                                healthy: false,
                                population: ServerPopulation::Easy,
                            }
                        },
                        Err(e) => {
                            error!(error = %e, "Error when trying to check gameserver");
                            continue;
                        },
                    };

                    server.update(status);
                }

                let mut server_list = server_copy.write().await;
                let _ = mem::replace(&mut *server_list, servers);
                drop(server_list);
                tokio::time::sleep(poll_interval).await;
            }
        });

        AgentServerManager { servers, farms }
    }

    async fn request_server_status(server: &AgentServer) -> Result<ServerStatusReport, reqwest::Error> {
        let address = format!("http://{}/status", server.rpc_address);
        let response = reqwest::get(&address).await?;
        response.json::<ServerStatusReport>().await
    }

    pub(crate) async fn servers(&self) -> Vec<AgentServer> {
        let servers = self.servers.read().await;
        servers.clone()
    }

    pub(crate) async fn server_details(&self, server_id: u16) -> Option<SocketAddr> {
        let servers = self.servers.read().await;
        servers
            .iter()
            .find(|server| server.id == server_id)
            .map(|server| server.address)
    }

    pub(crate) async fn reserve(
        &self,
        user_id: u32,
        username: &str,
        server_id: u16,
    ) -> Result<ReserveResponse, reqwest::Error> {
        let servers = self.servers.read().await;
        let server = match servers.iter().find(|server| server.id == server_id) {
            Some(s) => s,
            _ => return Ok(ReserveResponse::NotFound),
        };
        let token = server.token.clone();
        let address = server.rpc_address;
        drop(servers);
        let request = Client::new()
            .post(&format!("http://{}/request", address))
            .header("TOKEN", token)
            .json(&ReserveRequest {
                user_id,
                username: String::from(username),
            })
            .send()
            .await?;
        request.json().await
    }

    pub(crate) fn farms(&self) -> &Vec<Farm> {
        &self.farms
    }
}
