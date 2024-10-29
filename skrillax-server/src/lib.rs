use kanal::{unbounded, AsyncReceiver, AsyncSender, ReceiveError, Receiver, SendError, Sender};
use skrillax_stream::handshake::ActiveSecuritySetup;
use skrillax_stream::packet::AsPacket;
use skrillax_stream::stream::{InStreamError, OutStreamError, SilkroadStreamRead, SilkroadStreamWrite, SilkroadTcpExt};
use skrillax_stream::InputProtocol;
use std::io::{self, ErrorKind};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{instrument, trace, warn};

static STREAM_IDENTIFIER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct Connection<I: InputProtocol> {
    remote_addr: SocketAddr,
    identifier: u64,
    inbound: Receiver<I::Proto>,
    outbound: Sender<Box<dyn AsPacket + Send>>,
}

impl<I: InputProtocol + Send + 'static> Connection<I> {
    pub fn next(&self) -> Result<Option<I::Proto>, ReceiveError> {
        self.inbound.try_recv()
    }

    pub async fn next_async(&self) -> Result<I::Proto, ReceiveError> {
        self.inbound.as_async().recv().await
    }

    pub fn send<S: AsPacket + Send + 'static>(&self, packet: S) -> Result<(), SendError> {
        self.outbound.send(Box::new(packet))
    }

    pub fn id(&self) -> u64 {
        self.identifier
    }

    pub fn remote_address(&self) -> SocketAddr {
        self.remote_addr
    }

    #[instrument(skip(socket, inbound, outbound))]
    async fn handle(
        socket: TcpStream,
        identifier: u64,
        cancel: CancellationToken,
        inbound: Sender<I::Proto>,
        outbound: Receiver<Box<dyn AsPacket + Send>>,
    ) {
        let (mut reader, mut writer) = socket.into_silkroad_stream();
        if let Err(err) = ActiveSecuritySetup::handle(&mut reader, &mut writer).await {
            warn!(%err, "Failed to finish handshake.");
            return;
        }

        let outbound = outbound.to_async();
        let inbound = inbound.to_async();
        let send_cancel = cancel.clone();
        tokio::spawn(Self::handle_send(writer, outbound, identifier, send_cancel));
        tokio::spawn(Self::handle_receive(reader, inbound, identifier, cancel));
    }

    #[instrument(skip(writer, oubound_receiver, cancel))]
    async fn handle_send(
        mut writer: SilkroadStreamWrite<OwnedWriteHalf>,
        oubound_receiver: AsyncReceiver<Box<dyn AsPacket + Send>>,
        identifier: u64,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return;
                }
                recv = oubound_receiver.recv() => {
                    let Ok(packet) = recv else {
                        return;
                    };

                    let p = packet.as_packet();
                    drop(packet);
                    match writer.write(p).await {
                        Ok(_) => {},
                        Err(OutStreamError::IoError(io_error)) => {
                            cancel.cancel();
                            if matches!(io_error.kind(), ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset) {
                                trace!(id = identifier, "Connection was closed by the peer.");
                            } else {
                                warn!(id = identifier, %io_error, "Encountered some I/O error in connection.");
                            }
                            return;
                        }
                        Err(OutStreamError::Framing(_)) => {
                            warn!(id = identifier, "Tried to send an encrypted packet, but encryption was not set up.");
                        }
                    }
                }
            }
        }
    }

    #[instrument(skip(reader, inbound_sender, cancel))]
    async fn handle_receive(
        mut reader: SilkroadStreamRead<OwnedReadHalf>,
        inbound_sender: AsyncSender<I::Proto>,
        identifier: u64,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return;
                }
                recv = reader.next_packet::<I>() => {
                    match recv {
                        Ok(packet) => {
                            let to_send = inbound_sender.send(packet);
                            if let Err(_) = to_send.await {
                                return;
                            }
                        },
                        Err(InStreamError::EndOfStream) => {
                            return;
                        },
                        Err(InStreamError::UnmatchedOpcode(opcode)) => {
                            warn!(opcode, "Encountered unknown opcode.");
                            continue;
                        },
                        Err(other) => {
                            warn!(%other, "Unexpected error occurred.");
                            cancel.cancel();
                            return;
                        }
                    }
                }
            }
        }
    }
}

struct AsyncServerRunner<I: InputProtocol + Send + 'static> {
    token: CancellationToken,
    stream_receiver: Receiver<Connection<I>>,
    handle: JoinHandle<()>,
}

impl<I: InputProtocol + Send> AsyncServerRunner<I> {
    async fn run(listener: TcpListener, cancel_token: CancellationToken, connection_sender: Sender<Connection<I>>) {
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

                            tokio::spawn(async move {
                                Connection::<I>::handle(socket, identifier, child, inbound_sender, outbound_receiver).await;
                            });

                            if let Err(e) = connection_sender.send(connection) {
                                warn!(%e, "Could not send client over.");
                            }
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

pub struct Server<I: InputProtocol + Send + 'static> {
    listen_addr: SocketAddr,
    async_connector: AsyncServerRunner<I>,
}

impl<I: InputProtocol + Send + 'static> Server<I> {
    pub fn new(addr: SocketAddr) -> Result<Self, io::Error> {
        let (sender, receiver) = unbounded();
        let cancel = CancellationToken::new();

        let inner_cancel = cancel.clone();
        let socket = TcpSocket::new_v4()?;
        socket.bind(addr)?;
        let listener = socket.listen(1024)?;
        let join_handle =
            tokio::spawn(async move { AsyncServerRunner::<I>::run(listener, inner_cancel, sender).await });

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

    pub fn accepted_connections(&self) -> AcceptedClients<I> {
        AcceptedClients {
            inner: &self.async_connector.stream_receiver,
        }
    }

    pub async fn await_client(&self) -> Connection<I> {
        self.async_connector.stream_receiver.as_async().recv().await.unwrap()
    }
}

pub struct AcceptedClients<'a, I: InputProtocol> {
    inner: &'a Receiver<Connection<I>>,
}

impl<I: InputProtocol> Iterator for AcceptedClients<'_, I> {
    type Item = Connection<I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.try_recv().unwrap_or_else(|_| None)
    }
}
