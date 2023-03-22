use crate::stream::Stream;
use crossbeam_channel::Receiver;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

#[derive(Clone)]
pub struct SilkroadServer {
    stream_receiver: Receiver<Stream>,
    shutdown_token: CancellationToken,
}

impl SilkroadServer {
    async fn listen(
        runtime: Arc<Runtime>,
        socket: SocketAddr,
        cancel: CancellationToken,
    ) -> std::io::Result<Receiver<Stream>> {
        let listener = TcpListener::bind(socket).await?;
        let (stream_sender, stream_receiver) = crossbeam_channel::unbounded();
        let listen_cancel = cancel.clone();
        let inner_runtime = runtime.clone();
        runtime.spawn(async move {
            while let Some(connection) = tokio::select! {
                connected = listener.accept() => Some(connected),
                _ = listen_cancel.cancelled() => None
            } {
                if let Ok((socket, addr)) = connection {
                    debug!(?addr, "Accepted client");
                    let stream_sender = stream_sender.clone();
                    let socket_cancel = cancel.clone();
                    inner_runtime.spawn(async move {
                        // TODO include cancel token
                        match Stream::accept(socket).await {
                            Ok(stream) => {
                                stream_sender
                                    .send(stream)
                                    .expect("Accepted connection and receiver was closed.");
                            },
                            Err(_) if socket_cancel.is_cancelled() => {
                                // Connections are no longer accepted so we expect currently pending
                                // connections to error, but we no longer care.
                            },
                            Err(err) => {
                                error!(?addr, "Error in handshake: {:?}", err);
                            },
                        }
                    });
                }
            }
            debug!("Finished network")
        });
        Ok(stream_receiver)
    }

    pub fn new(runtime: Arc<Runtime>, listen: SocketAddr) -> Result<SilkroadServer, std::io::Error> {
        let shutdown_token = CancellationToken::new();
        let inner_runtime = runtime.clone();
        let inner_token = shutdown_token.clone();
        let stream_receiver =
            runtime.block_on(async move { Self::listen(inner_runtime, listen, inner_token).await })?;

        Ok(SilkroadServer {
            stream_receiver,
            shutdown_token,
        })
    }

    pub fn connected(&self) -> impl Iterator<Item = Stream> + '_ {
        self.stream_receiver.try_iter()
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel()
    }
}
