use crate::login::{LoginProvider, LoginResult};
use crate::patch::PatchInformation;
use crate::{AgentServerManager, NewsCacheAsync, Patcher};
use chrono::{TimeZone, Utc};
use silkroad_network::sid::StreamId;
use silkroad_network::stream::{Stream, StreamError, StreamReader, StreamWriter};
use silkroad_protocol::general::{IdentityInformation, ServerInfoSeed, ServerStateSeed};
use silkroad_protocol::login::{
    BlockReason, GatewayNotice, GatewayNoticeResponse, LoginResponse, PasscodeAccountStatus, PasscodeRequiredCode,
    PasscodeRequiredResponse, PasscodeResponse, PatchError, PatchResponse, PatchResult, PingServer, PingServerResponse,
    SecurityCodeResponse, SecurityError, ShardListResponse,
};
use silkroad_protocol::{ClientPacket, ServerPacket};
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
                        writer
                            .send(ServerPacket::PatchResponse(PatchResponse::new(PatchResult::UpToDate {
                                unknown: 0,
                            })))
                            .await?;
                    },
                    PatchInformation::RequiredFiles(files) => {
                        let response = PatchResponse::new(PatchResult::Problem {
                            error: PatchError::Update {
                                server_ip: "localhost".to_string(),
                                server_port: 80,
                                current_version: patcher.current_version(),
                                patch_files: files,
                                http_server: patcher.patch_host(),
                            },
                        });
                        writer.send(ServerPacket::PatchResponse(response)).await?;
                    },
                    PatchInformation::Outdated => {
                        let response = PatchResponse::new(PatchResult::Problem {
                            error: PatchError::InvalidVersion,
                        });
                        writer.send(ServerPacket::PatchResponse(response)).await?;
                    },
                },
                ClientPacket::IdentityInformation(identity) => {
                    debug!(?id, module = ?identity.module_name, local = identity.locality, "Client application identity");
                    writer
                        .send(ServerPacket::IdentityInformation(IdentityInformation {
                            module_name: "GatewayServer".to_string(),
                            locality: 0,
                        }))
                        .await?;
                    writer
                        .send(ServerPacket::ServerInfoSeed(ServerInfoSeed::new(0x1056)))
                        .await?; // TODO: need to figure out what this value actually represents.
                                 //   This seems like "server version", maybe expose a setting for it.
                    writer
                        .send(ServerPacket::ServerStateSeed(ServerStateSeed::new()))
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
                    let _ = writer
                        .send(ServerPacket::GatewayNoticeResponse(GatewayNoticeResponse::new(news)))
                        .await?;
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
                            let _ = writer
                                .send(ServerPacket::PasscodeRequiredResponse(PasscodeRequiredResponse::new(
                                    PasscodeRequiredCode::PasscodeRequired,
                                )))
                                .await?;
                        },
                        LoginResult::InvalidCredentials => {
                            let _ = writer
                                .send(ServerPacket::LoginResponse(LoginResponse {
                                    result: silkroad_protocol::login::LoginResult::Error {
                                        error: SecurityError::InvalidCredentials {
                                            max_attempts: 5,
                                            current_attempts: 1,
                                        },
                                    },
                                }))
                                .await?;
                        },
                        LoginResult::Blocked => {
                            let response = LoginResponse::new(silkroad_protocol::login::LoginResult::Error {
                                error: SecurityError::Blocked {
                                    reason: BlockReason::Punishment {
                                        reason: "You have been blocked.".to_string(),
                                        end: Utc.ymd(2099, 12, 31).and_hms(23, 59, 59),
                                    },
                                },
                            });
                            let _ = writer.send(ServerPacket::LoginResponse(response)).await?;
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
                                    let _ = writer
                                        .send(ServerPacket::PasscodeResponse(PasscodeResponse::new(2, 1)))
                                        .await;
                                    continue;
                                },
                            };

                        let result = login_provider
                            .try_login_passcode(&previous.username, &previous.password, &decoded_passcode)
                            .await;

                        match result {
                            LoginResult::Success(id) => {
                                writer
                                    .send(ServerPacket::SecurityCodeResponse(SecurityCodeResponse {
                                        account_status: PasscodeAccountStatus::Ok,
                                        result: 1,
                                        invalid_attempts: 3,
                                    }))
                                    .await?;
                                Self::try_reserve_spot(&mut writer, &agent_servers, id as u32, previous).await?
                            },
                            LoginResult::MissingPasscode => {
                                error!("Player entered passcode but we somehow didn't use it.");
                            },
                            LoginResult::InvalidCredentials => {
                                writer
                                    .send(ServerPacket::PasscodeRequiredResponse(PasscodeRequiredResponse {
                                        result: PasscodeRequiredCode::PasscodeInvalid,
                                    }))
                                    .await?;
                            },
                            LoginResult::Blocked => {
                                writer
                                    .send(ServerPacket::PasscodeRequiredResponse(PasscodeRequiredResponse {
                                        result: PasscodeRequiredCode::PasscodeBlocked,
                                    }))
                                    .await?;
                            },
                        }
                    }
                },
                ClientPacket::ShardListRequest(_) => {
                    let servers = agent_servers.servers().await;
                    let shards = servers.into_iter().map(|server| server.into()).collect();
                    let farms = agent_servers.farms().clone();

                    let response = ServerPacket::ShardListResponse(ShardListResponse { farms, shards });
                    let _ = writer.send(response).await?;
                },
                ClientPacket::PingServerRequest(_) => {
                    let ping_response = PingServerResponse {
                        servers: vec![PingServer {
                            index: 1,
                            domain: "localhost".to_string(),
                            unknown_1: 0x32,
                            unknown_2: 0xbd,
                        }],
                    };
                    let _ = writer.send(ServerPacket::PingServerResponse(ping_response)).await?;
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
        let result = agent_servers
            .reserve(user_id, &last_credentials.username, last_credentials.shard)
            .await;

        match result {
            None => {
                let _ = writer
                    .send(ServerPacket::LoginResponse(LoginResponse {
                        result: silkroad_protocol::login::LoginResult::Error {
                            error: SecurityError::Inspection,
                        },
                    }))
                    .await?;
            },
            Some(result) => {
                let _ = match result {
                    ReserveResponse::Success { ip, port, token, .. } => {
                        debug!("Got a spot at {}:{}: {}", ip, port, token);
                        writer
                            .send(ServerPacket::LoginResponse(LoginResponse {
                                result: silkroad_protocol::login::LoginResult::Success {
                                    session_id: token,
                                    agent_ip: ip,
                                    agent_port: port,
                                    unknown: 1,
                                },
                            }))
                            .await?
                    },
                    _ => {
                        writer
                            .send(ServerPacket::LoginResponse(LoginResponse {
                                result: silkroad_protocol::login::LoginResult::Error {
                                    error: SecurityError::ServerFull,
                                },
                            }))
                            .await?
                    },
                };
            },
        }
        Ok(())
    }
}
