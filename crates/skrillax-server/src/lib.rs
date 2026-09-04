use kanal::{unbounded, AsyncReceiver, AsyncSender, ReceiveError, Receiver, SendError, Sender};
use skrillax_stream::handshake::ActiveSecuritySetup;
use skrillax_stream::packet::SerdeContext;
use skrillax_stream::registry::PacketRegistry;
use skrillax_stream::stream::{
    DynamicPacket, InStreamError, OutStreamError, SilkroadStreamRead, SilkroadStreamWrite, SilkroadTcpExt,
};
use std::fmt::Debug;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, instrument, trace, warn};

static STREAM_IDENTIFIER: AtomicU64 = AtomicU64::new(1);

type ContextInitializer = Arc<dyn Fn(&SerdeContext) + Send + Sync>;

#[derive(Clone)]
pub struct Connection {
    remote_addr: SocketAddr,
    identifier: u64,
    inbound: Receiver<DynamicPacket>,
    outbound: Sender<DynamicPacket>,
}

impl Connection {
    pub fn next(&self) -> Result<Option<DynamicPacket>, ReceiveError> {
        self.inbound.try_recv()
    }

    pub async fn next_async(&self) -> Result<DynamicPacket, ReceiveError> {
        self.inbound.as_async().recv().await
    }

    pub fn send<S: Into<DynamicPacket> + Debug>(&self, packet: S) -> Result<(), SendError> {
        debug!("Sending packet: {:?}", packet);
        self.outbound.send(packet.into())
    }

    pub fn id(&self) -> u64 {
        self.identifier
    }

    pub fn remote_address(&self) -> SocketAddr {
        self.remote_addr
    }

    #[instrument(skip(socket, registry, inbound, outbound, cancel, initialize_context))]
    async fn handle(
        socket: TcpStream,
        identifier: u64,
        registry: PacketRegistry,
        cancel: CancellationToken,
        inbound: Sender<DynamicPacket>,
        outbound: Receiver<DynamicPacket>,
        initialize_context: ContextInitializer,
    ) -> bool {
        let (mut reader, mut writer) = socket.into_silkroad_stream(registry);
        initialize_context(&reader.context());
        if let Err(err) = ActiveSecuritySetup::handle(&mut reader, &mut writer).await {
            warn!(%err, "Failed to finish handshake.");
            return false;
        }

        let outbound = outbound.to_async();
        let inbound = inbound.to_async();
        let send_cancel = cancel.clone();
        tokio::spawn(Self::handle_send(writer, outbound, identifier, send_cancel));
        tokio::spawn(Self::handle_receive(reader, inbound, identifier, cancel));
        true
    }

    #[instrument(skip(writer, outbound_receiver, cancel))]
    async fn handle_send(
        mut writer: SilkroadStreamWrite<OwnedWriteHalf>,
        outbound_receiver: AsyncReceiver<DynamicPacket>,
        identifier: u64,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return;
                }
                recv = outbound_receiver.recv() => {
                    let Ok(packet) = recv else {
                        return;
                    };

                    match writer.write_packet(packet).await {
                        Ok(_) => {},
                        Err(OutStreamError::IoError(io_error)) => {
                            cancel.cancel();
                            if matches!(io_error.kind(), ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset) {
                                trace!(identifier, "Connection was closed by the peer.");
                            } else {
                                warn!(identifier, %io_error, "Encountered some I/O error in connection.");
                            }
                            return;
                        },
                        Err(OutStreamError::Framing(_)) => {
                            warn!(identifier, "Tried to send an encrypted packet, but encryption was not set up.");
                        },
                        Err(OutStreamError::UnknownOpcode(_)) => {
                            warn!(identifier, "Tried to send an unknown opcode.");
                        },
                        Err(OutStreamError::PacketError(e)) => {
                            warn!(identifier, %e);
                        },
                        Err(OutStreamError::DynamicPacketType(e)) => {
                            warn!(identifier, %e);
                        }
                    }
                }
            }
        }
    }

    #[instrument(skip(reader, inbound_sender, cancel))]
    async fn handle_receive(
        mut reader: SilkroadStreamRead<OwnedReadHalf>,
        inbound_sender: AsyncSender<DynamicPacket>,
        identifier: u64,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return;
                }
                recv = reader.next_packet() => {
                    match recv {
                        Ok(packet) => {
                            let to_send = inbound_sender.send(packet);
                            if to_send.await.is_err() {
                                return;
                            }
                        },
                        Err(InStreamError::EndOfStream) => {
                            debug!("Disconnected.");
                            return;
                        },
                        Err(InStreamError::UnmatchedOpcode(opcode)) => {
                            warn!(opcode, "Encountered unknown opcode.");
                            continue;
                        },
                        Err(other) => {
                            warn!(error = %other, "Unexpected error occurred.");
                            cancel.cancel();
                            return;
                        }
                    }
                }
            }
        }
    }
}

struct AsyncServerRunner {
    token: CancellationToken,
    stream_receiver: Receiver<Connection>,
    handle: JoinHandle<()>,
}

impl AsyncServerRunner {
    async fn run(
        listener: TcpListener,
        packet_registry: PacketRegistry,
        cancel_token: CancellationToken,
        connection_sender: Sender<Connection>,
        initialize_context: ContextInitializer,
    ) {
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => break,
                accepted = listener.accept() => {
                    match accepted {
                        Ok((socket, addr)) => {
                            let identifier = STREAM_IDENTIFIER.fetch_add(1, Ordering::SeqCst);
                            let (inbound_sender, inbound_receiver) = unbounded();
                            let (outbound_sender, outbound_receiver) = unbounded();
                            let connection = Connection {
                                remote_addr: addr,
                                identifier,
                                inbound: inbound_receiver,
                                outbound: outbound_sender
                            };

                            let child = cancel_token.child_token();

                            let packet_registry = packet_registry.clone();
                            let connection_sender = connection_sender.clone();
                            let initialize_context = initialize_context.clone();
                            tokio::spawn(async move {
                                if Connection::handle(
                                    socket,
                                    identifier,
                                    packet_registry,
                                    child,
                                    inbound_sender,
                                    outbound_receiver,
                                    initialize_context,
                                ).await {
                                    if let Err(e) = connection_sender.send(connection) {
                                        warn!(%e, "Could not send client over.");
                                    }
                                }
                            });
                            continue;
                        },
                        Err(e) => {
                            warn!(%e, "Could not accept client.")
                        }
                    }
                }
            }
        }
    }
}

pub struct Server {
    listen_addr: SocketAddr,
    async_connector: AsyncServerRunner,
}

impl Server {
    pub fn new(addr: SocketAddr, packet_registry: PacketRegistry) -> Result<Self, io::Error> {
        Self::new_with_context_initializer(addr, packet_registry, |_| {})
    }

    /// Creates a server that initializes each connection's serialization context
    /// before its handshake or packet-processing tasks begin.
    pub fn new_with_context_initializer<F>(
        addr: SocketAddr,
        packet_registry: PacketRegistry,
        initialize_context: F,
    ) -> Result<Self, io::Error>
    where
        F: Fn(&SerdeContext) + Send + Sync + 'static,
    {
        let (sender, receiver) = unbounded();
        let cancel = CancellationToken::new();

        let inner_cancel = cancel.clone();
        let socket = TcpSocket::new_v4()?;
        socket.bind(addr)?;
        let listener = socket.listen(1024)?;
        let initialize_context = Arc::new(initialize_context);
        let join_handle = tokio::spawn(async move {
            AsyncServerRunner::run(listener, packet_registry, inner_cancel, sender, initialize_context).await
        });

        Ok(Self {
            listen_addr: addr,
            async_connector: AsyncServerRunner {
                token: cancel,
                handle: join_handle,
                stream_receiver: receiver,
            },
        })
    }

    pub fn is_running(&self) -> bool {
        !self.async_connector.handle.is_finished()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn stop(&self) {
        self.async_connector.token.cancel();
    }

    pub fn accepted_connections(&self) -> AcceptedClients<'_> {
        AcceptedClients {
            inner: &self.async_connector.stream_receiver,
        }
    }

    pub async fn await_client(&self) -> Connection {
        self.async_connector.stream_receiver.as_async().recv().await.unwrap()
    }
}

pub struct AcceptedClients<'a> {
    inner: &'a Receiver<Connection>,
}

impl Iterator for AcceptedClients<'_> {
    type Item = Connection;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.try_recv().unwrap_or_else(|_| None)
    }
}
