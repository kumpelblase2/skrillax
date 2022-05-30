use reqwest::Client;
use silkroad_protocol::login::Shard;
use silkroad_rpc::{ReserveRequest, ReserveResponse, ServerPopulation, ServerStatusReport};
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

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
    pub(crate) address: String,
    pub(crate) region: ServerRegion,
    pub(crate) status: ServerStatus,
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
    pub(crate) fn new(id: u16, name: String, address: String, region: ServerRegion) -> Self {
        AgentServer {
            id,
            name,
            address,
            region,
            status: ServerStatus::Offline,
            population: ServerPopulation::Easy,
        }
    }

    pub(crate) fn update(&mut self, report: ServerStatusReport) {
        self.status = if report.healthy {
            if matches!(&self.status, ServerStatus::Offline) {
                info!(server = ?self.name, "Status changed to online.");
            }
            ServerStatus::Online
        } else {
            if matches!(&self.status, ServerStatus::Online) {
                info!(server = ?self.name, "Status changed to offline.");
            }
            ServerStatus::Offline
        };

        if self.population != report.population {
            info!(server = ?self.name, "Population changed from {} to {}.", &self.population, &report.population);
        }
        self.population = report.population;
    }
}

#[derive(Clone)]
pub(crate) struct AgentServerManager {
    servers: Arc<RwLock<Vec<AgentServer>>>,
}

impl AgentServerManager {
    pub(crate) fn new(poll_interval: Duration) -> Self {
        let servers: Arc<RwLock<Vec<AgentServer>>> = Arc::new(RwLock::new(Vec::new()));
        let server_copy = servers.clone();

        tokio::spawn(async move {
            loop {
                let servers = server_copy.read().await;
                let mut server_list = servers.clone();
                drop(servers);
                for server in server_list.iter_mut() {
                    let address = &format!("http://{}/status", server.address);
                    let status = match reqwest::get(address).await {
                        Ok(resp) => resp.json::<ServerStatusReport>().await.unwrap(),
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

                let mut servers = server_copy.write().await;
                servers.swap_with_slice(&mut server_list);
                drop(servers);
                tokio::time::sleep(poll_interval).await;
            }
        });

        AgentServerManager { servers }
    }

    pub(crate) async fn add_server(&self, server: AgentServer) {
        let mut servers = self.servers.write().await;
        debug!("Added {} at {} to the server list.", &server.name, &server.address);
        servers.push(server);
    }

    pub(crate) async fn servers(&self) -> Vec<AgentServer> {
        let servers = self.servers.read().await;
        servers.clone()
    }

    pub(crate) async fn reserve(&self, user_id: u32, username: &str, server_id: u16) -> Option<ReserveResponse> {
        let servers = self.servers.read().await;
        let server = servers.iter().find(|server| server.id == server_id)?;
        let address = server.address.clone();
        drop(servers);
        let client = Client::new();
        return match client
            .post(&format!("http://{}/request", address))
            .json(&ReserveRequest {
                user_id,
                username: String::from(username),
            })
            .send()
            .await
        {
            Ok(response) => Some(response.json().await.unwrap()),
            _ => None,
        };
    }
}
