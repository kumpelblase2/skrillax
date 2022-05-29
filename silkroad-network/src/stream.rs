use crate::codec::{SilkroadFrameDecoder, SilkroadFrameEncoder};
use crate::frame::{FrameError, SilkroadFrame};
use crate::security_setup::{HandshakeError, SecurityHandshake};
use crate::sid::StreamId;
use crossbeam_channel::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use silkroad_protocol::error::ProtocolError;
use silkroad_protocol::{ClientPacket, ServerPacket};
use silkroad_security::security::SilkroadSecurity;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, trace_span};

pub type StreamResult<T> = Result<T, StreamError>;
pub type SendResult = StreamResult<()>;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Handshake did not complete")]
    IncompleteHandshake,
    #[error("A frame level error occurred when sending a packet")]
    FrameError(#[from] FrameError),
    #[error("Error occurred at the protocol level")]
    ProtocolError(#[from] ProtocolError),
    #[error("Stream has been closed")]
    StreamClosed,
    #[error("A massive packet was previously sent and is expecting {0} packets to follow")]
    UnconsumedMassivePacket(u16),
}

type SilkroadFramedRead = FramedRead<OwnedReadHalf, SilkroadFrameDecoder>;
type SilkroadFramedWrite = FramedWrite<OwnedWriteHalf, SilkroadFrameEncoder>;

pub struct StreamReader {
    id: StreamId,
    inner: SilkroadFramedRead,
    massive_packet: Option<(u16, u16)>,
}

pub struct StreamWriter {
    id: StreamId,
    inner: SilkroadFramedWrite,
}

impl StreamReader {
    pub fn new(id: StreamId, reader: SilkroadFramedRead) -> Self {
        StreamReader {
            id,
            inner: reader,
            massive_packet: None,
        }
    }

    pub async fn start_loop(reader: Self, writer: Sender<ClientPacket>) {
        let mut reader = reader;
        loop {
            match reader.next().await {
                Ok(packet) => match writer.send(packet) {
                    Ok(_) => {}
                    Err(_) => return,
                },
                Err(e) => {
                    tracing::warn!("Could not parse frame :( {:?}", e);
                    return;
                }
            }
        }
    }

    pub async fn next(&mut self) -> StreamResult<ClientPacket> {
        while let Some(packet) = self.inner.next().await {
            match packet {
                Ok(frame) => match frame {
                    SilkroadFrame::Packet { data, opcode, .. } => {
                        let span = trace_span!("decoding", id = ?self.id);
                        let _enter = span.enter();
                        return Ok(ClientPacket::deserialize(opcode, data)?);
                    }
                    SilkroadFrame::MassiveHeader {
                        contained_count,
                        contained_opcode,
                        ..
                    } => {
                        if let Some((_, remaining)) = &self.massive_packet {
                            return Err(StreamError::UnconsumedMassivePacket(*remaining));
                        }
                        self.massive_packet = Some((contained_opcode, contained_count));
                    }
                    SilkroadFrame::MassiveContainer { inner, .. } => {
                        return match &self.massive_packet {
                            Some((opcode, count)) => {
                                let span = trace_span!("decoding", id = ?self.id);
                                let _enter = span.enter();
                                let result = ClientPacket::deserialize(*opcode, inner)?;
                                let new_count = *count - 1;
                                if new_count > 0 {
                                    self.massive_packet = Some((*opcode, new_count));
                                } else {
                                    self.massive_packet = None;
                                }
                                Ok(result)
                            }
                            None => Err(StreamError::ProtocolError(
                                ProtocolError::StrayMassivePacket,
                            )),
                        };
                    }
                },
                Err(f) => {
                    return Err(f.into());
                }
            }
        }
        Err(StreamError::StreamClosed)
    }
}

impl StreamWriter {
    pub fn new(id: StreamId, writer: SilkroadFramedWrite) -> Self {
        StreamWriter { id, inner: writer }
    }

    pub async fn start_loop(writer: Self, receiver: UnboundedReceiver<ServerPacket>) {
        let mut writer = writer;
        let mut receiver = receiver;
        while let Some(packet) = receiver.recv().await {
            match writer.send(packet).await {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }

    pub async fn send(&mut self, packet: ServerPacket) -> SendResult {
        let span = trace_span!("encoding", id = ?self.id);
        let enter = span.enter();
        let frames = SilkroadFrame::create_for(packet);
        drop(enter);

        let mut iter = futures::stream::iter(frames.into_iter().map(Ok));
        self.inner.send_all(&mut iter).await?;

        Ok(())
    }
}

pub struct Stream {
    id: StreamId,
    receiver: Receiver<ClientPacket>,
    sender: tokio::sync::mpsc::UnboundedSender<ServerPacket>,
}

impl Stream {
    pub async fn accept(conn: TcpStream) -> Result<Stream, HandshakeError> {
        Self::accept_with_enc(conn, true).await
    }

    pub async fn accept_with_enc(
        conn: TcpStream,
        enable_encryption: bool,
    ) -> Result<Stream, HandshakeError> {
        let id = StreamId::new();
        let (writer, reader) = Self::init_stream(id, conn, enable_encryption).await?;

        let (writer_write, writer_receive) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(StreamWriter::start_loop(writer, writer_receive));

        let (reader_write, reader_read) = crossbeam_channel::unbounded();
        tokio::spawn(StreamReader::start_loop(reader, reader_write));

        Ok(Stream {
            id,
            receiver: reader_read,
            sender: writer_write,
        })
    }

    pub async fn init_stream(
        id: StreamId,
        conn: TcpStream,
        enable_encryption: bool,
    ) -> Result<(StreamWriter, StreamReader), HandshakeError> {
        let (read, write) = conn.into_split();
        let security = if enable_encryption {
            Some(Arc::new(RwLock::new(SilkroadSecurity::default())))
        } else {
            None
        };

        let mut writer = StreamWriter::new(
            id,
            FramedWrite::new(write, SilkroadFrameEncoder::new(security.clone())),
        );
        let mut reader = StreamReader::new(
            id,
            FramedRead::new(read, SilkroadFrameDecoder::new(security.clone())),
        );

        debug!(?id, "Starting handshake");
        if enable_encryption {
            SecurityHandshake::do_handshake(&mut writer, &mut reader, security).await?;
        }
        Ok((writer, reader))
    }

    pub fn has_activity(&self) -> bool {
        !self.receiver.is_empty()
    }

    pub fn received(&self) -> Result<Option<ClientPacket>, StreamError> {
        match self.receiver.try_recv() {
            Ok(p) => Ok(Some(p)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            _ => Err(StreamError::StreamClosed),
        }
    }

    pub fn send<P>(&self, operation: P) -> SendResult
    where
        P: Into<ServerPacket>,
    {
        self.sender
            .send(operation.into())
            .map_err(|_| StreamError::StreamClosed)
    }

    pub fn id(&self) -> &StreamId {
        &self.id
    }

    pub fn is_disconnected(&self) -> bool {
        self.sender.is_closed()
    }
}

impl PartialEq for Stream {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
