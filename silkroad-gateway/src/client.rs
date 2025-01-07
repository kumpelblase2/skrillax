use crate::login::{LoginProvider, LoginResult};
use crate::passcode::PasscodeDecoder;
use crate::patch::PatchInformation;
use crate::protocol::GatewayClientProtocol;
use crate::{AgentServerManager, NewsCacheAsync, Patcher};
use chrono::{TimeZone, Utc};
use color_eyre::Result;
use silkroad_gateway_protocol::*;
use silkroad_rpc::ReserveResponse;
use skrillax_server::Connection;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

struct LastCredentials {
    username: String,
    password: String,
    shard: u16,
}

pub(crate) struct Client;

impl Client {
    pub(crate) async fn handle_client(
        connection: Connection<GatewayClientProtocol>,
        cancel: CancellationToken,
        news: Arc<Mutex<NewsCacheAsync>>,
        patcher: Arc<Patcher>,
        login_provider: Arc<LoginProvider>,
        agent_servers: AgentServerManager,
    ) -> Result<()> {
        let mut last_credentials = None;
        loop {
            tokio::select! {
                res = timeout(Duration::from_secs(10), connection.next_async()) => {
                    let Ok(packet) = res else {
                        break;
                    };

                    match packet?.as_ref() {
                        GatewayClientProtocol::KeepAlive(_) => {},
                        GatewayClientProtocol::PatchRequest(patch) => match patcher.get_patch_information(patch.version) {
                            PatchInformation::UpToDate => {
                                connection.send(PatchResponse::up_to_date())?;
                            },
                            PatchInformation::RequiresUpdate {
                                files,
                                target_version,
                                host,
                            } => {
                                let response = PatchResponse::error(PatchError::Update {
                                    server_ip: "localhost".to_string(),
                                    server_port: 80,
                                    current_version: target_version,
                                    patch_files: files,
                                    http_server: host,
                                });
                                connection.send(response)?;
                            },
                            PatchInformation::Outdated => {
                                connection.send(PatchResponse::error(PatchError::InvalidVersion))?;
                            },
                        },
                        GatewayClientProtocol::IdentityInformation(identity) => {
                            debug!(module = ?identity.module_name, local = identity.locality, "Client application identity");
                            connection.send(IdentityInformation {
                                module_name: "GatewayServer".to_string(),
                                locality: 0,
                            })?;
                        },
                        GatewayClientProtocol::GatewayNoticeRequest(_) => {
                            let mut news = news.lock().await;
                            let news = news.get_news().await;
                            let news = news
                                .iter()
                                .map(|news| GatewayNotice {
                                    subject: news.title.clone(),
                                    article: news.body.clone(),
                                    published: news.date,
                                })
                                .collect();
                            connection.send(GatewayNoticeResponse::new(news))?;
                        },
                        GatewayClientProtocol::LoginRequest(login) => {
                            last_credentials = Some(LastCredentials {
                                username: login.username.clone(),
                                password: login.password.clone(),
                                shard: login.shard_id,
                            });

                            match login_provider.try_login(&login.username, &login.password).await {
                                LoginResult::Success(id) => {
                                    let creds = last_credentials
                                        .as_ref()
                                        .expect("We just set the credentials so have to be present");
                                    Self::try_reserve_spot(&connection, &agent_servers, id as u32, creds).await?
                                },
                                LoginResult::MissingPasscode => {
                                    connection.send(PasscodeRequiredResponse::passcode_required())?;
                                },
                                LoginResult::InvalidCredentials => {
                                    connection.send(LoginResponse::error(SecurityError::InvalidCredentials {
                                        max_attempts: 5,
                                        current_attempts: 1,
                                    }))?;
                                },
                                LoginResult::Blocked => {
                                    let response = LoginResponse::error(SecurityError::Blocked {
                                        reason: BlockReason::Punishment {
                                            reason: "You have been blocked.".to_string(),
                                            end: Utc.with_ymd_and_hms(2099, 12, 31, 23, 59, 59).unwrap(),
                                        },
                                    });
                                    connection.send(response)?;
                                },
                            }
                        },
                        GatewayClientProtocol::SecurityCodeInput(input) => {
                            let previous = last_credentials.as_ref();
                            if let Some(previous) = previous {
                                let decoded_passcode =
                                    match PasscodeDecoder::get().decode_passcode(input.inner_size, &input.data) {
                                        Ok(passcode) => passcode,
                                        Err(_) => {
                                            // Maybe this should return a more fitting response code?
                                            // Or should the client just be ditched?
                                            connection.send(PasscodeResponse::new(2, 1))?;
                                            continue;
                                        },
                                    };

                                let result = login_provider
                                    .try_login_passcode(&previous.username, &previous.password, &decoded_passcode)
                                    .await;

                                match result {
                                    LoginResult::Success(id) => {
                                        connection.send(SecurityCodeResponse::success())?;
                                        Self::try_reserve_spot(&connection, &agent_servers, id as u32, previous).await?
                                    },
                                    LoginResult::MissingPasscode => {
                                        error!("Player entered passcode but we somehow didn't use it.");
                                    },
                                    LoginResult::InvalidCredentials => {
                                        connection.send(PasscodeRequiredResponse::passcode_invalid())?;
                                    },
                                    LoginResult::Blocked => {
                                        connection.send(PasscodeRequiredResponse::passcode_blocked())?;
                                    },
                                }
                            }
                        },
                        GatewayClientProtocol::ShardListRequest(_) => {
                            let servers = agent_servers.servers().await;
                            let shards = servers.into_iter().map(|server| server.into()).collect();
                            let farms = agent_servers.farms().clone();

                            connection.send(ShardListResponse { farms, shards })?;
                        },
                        GatewayClientProtocol::PingServerRequest(_) => {
                            let ping_response = PingServerResponse::new(vec![PingServer::new(1, "localhost".to_string())]);
                            connection.send(ping_response)?;
                        },
                    }
                }
                _ = cancel.cancelled() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn try_reserve_spot(
        connection: &Connection<GatewayClientProtocol>,
        agent_servers: &AgentServerManager,
        user_id: u32,
        last_credentials: &LastCredentials,
    ) -> Result<()> {
        let server = match agent_servers.server_details(last_credentials.shard).await {
            Some(addr) => addr,
            None => {
                connection.send(LoginResponse::error(SecurityError::Inspection))?;
                return Ok(());
            },
        };

        let result = agent_servers
            .reserve(user_id, &last_credentials.username, last_credentials.shard)
            .await;

        match result {
            Err(e) => {
                debug!(error = %e, "Error when reserving a spot");
                connection.send(LoginResponse::error(SecurityError::Inspection))?
            },
            Ok(result) => match result {
                ReserveResponse::Success { token, .. } => {
                    let ip = server.ip();
                    let port = server.port();
                    debug!("Got a spot at {ip}:{port}: {token}");
                    connection.send(LoginResponse {
                        result: silkroad_gateway_protocol::LoginResult::Success {
                            session_id: token,
                            agent_ip: ip.to_string(),
                            agent_port: port,
                            unknown: 1,
                        },
                    })?
                },
                ReserveResponse::NotFound => connection.send(LoginResponse::error(SecurityError::Inspection))?,
                ReserveResponse::Full => connection.send(LoginResponse::error(SecurityError::ServerFull))?,
                ReserveResponse::Duplicate => connection.send(LoginResponse::error(SecurityError::AlreadyConnected))?,
                ReserveResponse::Error(message) => {
                    debug!("Could not reserve spot: {message}");
                    connection.send(LoginResponse::error(SecurityError::ServerFull))?
                },
            },
        }
        Ok(())
    }
}
