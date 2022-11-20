use crate::login::{LoginProvider, LoginResult};
use crate::patch::PatchInformation;
use crate::{AgentServerManager, NewsCacheAsync, Patcher};
use chrono::{TimeZone, Utc};
use silkroad_network::sid::StreamId;
use silkroad_network::stream::{Stream, StreamError, StreamReader, StreamWriter};
use silkroad_protocol::general::IdentityInformation;
use silkroad_protocol::login::{
    BlockReason, GatewayNotice, GatewayNoticeResponse, LoginResponse, PasscodeRequiredResponse, PasscodeResponse,
    PatchError, PatchResponse, PingServer, PingServerResponse, SecurityCodeResponse, SecurityError, ShardListResponse,
};
use silkroad_protocol::ClientPacket;
use silkroad_rpc::ReserveResponse;
use silkroad_security::passcode::PassCodeDecoder;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace};

struct LastCredentials {
    username: String,
    password: String,
    shard: u16,
}

pub(crate) struct Client;

impl Client {
    pub(crate) async fn handle_client(
        id: StreamId,
        socket: TcpStream,
        cancel: CancellationToken,
        news: Arc<Mutex<NewsCacheAsync>>,
        patcher: Arc<Patcher>,
        login_provider: Arc<LoginProvider>,
        agent_servers: AgentServerManager,
    ) {
        match Stream::init_stream(id, socket, true).await {
            Ok((writer, reader)) => {
                match Self::handle_socket(id, reader, writer, news, patcher, login_provider, agent_servers).await {
                    Err(StreamError::StreamClosed) => {
                        trace!(?id, "Client connection closed");
                    },
                    Err(e) => {
                        debug!(?id, "Client disconnected: {:?}", e);
                    },
                    _ => {},
                }
            },
            Err(_) if cancel.is_cancelled() => {},
            Err(err) => {
                error!(?id, "Error in handshake: {:?}", err);
            },
        }
    }

    async fn handle_socket(
        id: StreamId,
        reader: StreamReader,
        writer: StreamWriter,
        news: Arc<Mutex<NewsCacheAsync>>,
        patcher: Arc<Patcher>,
        login_provider: Arc<LoginProvider>,
        agent_servers: AgentServerManager,
    ) -> Result<(), StreamError> {
        let mut reader = reader;
        let mut writer = writer;
        let mut last_credentials = None;
        while let Ok(packet) = timeout(Duration::from_secs(10), reader.next()).await {
            match packet? {
                ClientPacket::KeepAlive(_) => {},
                ClientPacket::PatchRequest(patch) => match patcher.get_patch_information(patch.version) {
                    PatchInformation::UpToDate => {
                        writer.send(PatchResponse::up_to_date()).await?;
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
                        writer.send(response).await?;
                    },
                    PatchInformation::Outdated => {
                        writer.send(PatchResponse::error(PatchError::InvalidVersion)).await?;
                    },
                },
                ClientPacket::IdentityInformation(identity) => {
                    debug!(?id, module = ?identity.module_name, local = identity.locality, "Client application identity");
                    writer
                        .send(IdentityInformation {
                            module_name: "GatewayServer".to_string(),
                            locality: 0,
                        })
                        .await?;
                },
                ClientPacket::GatewayNoticeRequest(_) => {
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
                    writer.send(GatewayNoticeResponse::new(news)).await?;
                },
                ClientPacket::LoginRequest(login) => {
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
                            Self::try_reserve_spot(&mut writer, &agent_servers, id as u32, creds).await?
                        },
                        LoginResult::MissingPasscode => {
                            writer.send(PasscodeRequiredResponse::passcode_required()).await?;
                        },
                        LoginResult::InvalidCredentials => {
                            writer
                                .send(LoginResponse::error(SecurityError::InvalidCredentials {
                                    max_attempts: 5,
                                    current_attempts: 1,
                                }))
                                .await?;
                        },
                        LoginResult::Blocked => {
                            let response = LoginResponse::error(SecurityError::Blocked {
                                reason: BlockReason::Punishment {
                                    reason: "You have been blocked.".to_string(),
                                    end: Utc.ymd(2099, 12, 31).and_hms(23, 59, 59),
                                },
                            });
                            writer.send(response).await?;
                        },
                    }
                },
                ClientPacket::SecurityCodeInput(input) => {
                    let previous = last_credentials.as_ref();
                    if let Some(previous) = previous {
                        let decoded_passcode =
                            match PassCodeDecoder::get().decode_passcode(input.inner_size, &input.data) {
                                Ok(passcode) => passcode,
                                Err(_) => {
                                    // Maybe this should return a more fitting response code?
                                    // Or should the client just be ditched?
                                    writer.send(PasscodeResponse::new(2, 1)).await?;
                                    continue;
                                },
                            };

                        let result = login_provider
                            .try_login_passcode(&previous.username, &previous.password, &decoded_passcode)
                            .await;

                        match result {
                            LoginResult::Success(id) => {
                                writer.send(SecurityCodeResponse::success()).await?;
                                Self::try_reserve_spot(&mut writer, &agent_servers, id as u32, previous).await?
                            },
                            LoginResult::MissingPasscode => {
                                error!("Player entered passcode but we somehow didn't use it.");
                            },
                            LoginResult::InvalidCredentials => {
                                writer.send(PasscodeRequiredResponse::passcode_invalid()).await?;
                            },
                            LoginResult::Blocked => {
                                writer.send(PasscodeRequiredResponse::passcode_blocked()).await?;
                            },
                        }
                    }
                },
                ClientPacket::ShardListRequest(_) => {
                    let servers = agent_servers.servers().await;
                    let shards = servers.into_iter().map(|server| server.into()).collect();
                    let farms = agent_servers.farms().clone();

                    writer.send(ShardListResponse { farms, shards }).await?;
                },
                ClientPacket::PingServerRequest(_) => {
                    let ping_response = PingServerResponse::new(vec![PingServer {
                        index: 1,
                        domain: "localhost".to_string(),
                        unknown_1: 0x32,
                        unknown_2: 0xbd,
                    }]);
                    writer.send(ping_response).await?;
                },
                _ => {},
            }
        }

        Ok(())
    }

    async fn try_reserve_spot(
        writer: &mut StreamWriter,
        agent_servers: &AgentServerManager,
        user_id: u32,
        last_credentials: &LastCredentials,
    ) -> Result<(), StreamError> {
        let server = match agent_servers.server_details(last_credentials.shard).await {
            Some(addr) => addr,
            None => {
                writer.send(LoginResponse::error(SecurityError::Inspection)).await?;
                return Ok(());
            },
        };

        let result = agent_servers
            .reserve(user_id, &last_credentials.username, last_credentials.shard)
            .await;

        match result {
            Err(e) => {
                debug!(client = ?writer.id(), error = %e, "Error when reserving a spot");
                writer.send(LoginResponse::error(SecurityError::Inspection)).await?
            },
            Ok(result) => match result {
                ReserveResponse::Success { token, .. } => {
                    let ip = server.ip();
                    let port = server.port();
                    debug!(client = ?writer.id(), "Got a spot at {ip}:{port}: {token}");
                    writer
                        .send(LoginResponse {
                            result: silkroad_protocol::login::LoginResult::Success {
                                session_id: token,
                                agent_ip: ip.to_string(),
                                agent_port: port,
                                unknown: 1,
                            },
                        })
                        .await?
                },
                ReserveResponse::NotFound => {
                    writer
                        .send(LoginResponse::error(SecurityError::AlreadyConnected))
                        .await?
                },
                ReserveResponse::Error(message) => {
                    debug!(client = ?writer.id(), "Could not reserve spot: {message}");
                    writer.send(LoginResponse::error(SecurityError::ServerFull)).await?
                },
            },
        }
        Ok(())
    }
}
