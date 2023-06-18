use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use silkroad_protocol::ServerPacket;
use silkroad_security::security::{SilkroadSecurity, SilkroadSecurityError};
use std::cmp::{max, min};
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tracing::trace_span;

const MASSIVE_PACKET_OPCODE: u16 = 0x600D;

/// A 'frame' denotes the most fundamental block of data that can be sent between
/// the client and the server. Any and all operations or data exchanges are built
/// on top of a frame.
///
/// There are two types of frames: a basic frame and a massive frame. The latter
/// is split into a frame for the header and a frame for the data. Only a basic
/// frame may be encrypted. All server bound frames also contain an encryption-based
/// counter to avoid replay attacks and a one byte CRC checksum for integrity
/// checks.
pub enum SilkroadFrame {
    /// The most basic frame containing exactly one operation identified
    /// by its opcode.
    Packet {
        count: u8,
        crc: u8,
        opcode: u16,
        encrypted: bool,
        /// The contained data. If this frame was marked encrypted, this
        /// already contains the decrypted data.
        data: Bytes,
    },
    /// The header portion of a massive packet which contains information
    /// that is necessary for the identification and usage of the followed
    /// [SilkroadFrame::MassiveContainer] frame.
    MassiveHeader {
        count: u8,
        crc: u8,
        contained_opcode: u16,
        contained_count: u16,
    },
    /// The data container portion of a massive packet. Must come after
    /// a [SilkroadFrame::MassiveHeader]. Given the opcode and included
    /// count specified in the header frame, contains the data for `n`
    /// operations of the same opcode.
    MassiveContainer { count: u8, crc: u8, inner: Bytes },
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("I/O error when reading/writing from/to stream")]
    IoError(#[from] std::io::Error),
    /// A frame always starts with its size. This error occurs if the provided
    /// buffer is not large enough given the spezified size.
    #[error("The frame has not been completely transmitted yet")]
    Incomplete,
    #[error("The frame is encrypted, but no security was provided")]
    MissingSecurity,
    #[error("Error when encrypting/decrypting the frame")]
    SecurityError(#[from] SilkroadSecurityError),
}

impl SilkroadFrame {
    fn find_next_encrypted_length(length: usize) -> usize {
        SilkroadSecurity::find_encrypted_length(length)
    }

    /// Creates the necessary frames for the given [ServerPacket]. This will either return
    /// a [Vec] of size `1` (if it's not a massive packet) or `1+n` (for massive packets), where
    /// `n` depends on the size of the data packets.
    pub fn create_for(packet: ServerPacket) -> Vec<SilkroadFrame> {
        if packet.is_massive() {
            let (opcode, mut bytes) = packet.into_serialize();
            let required_packets = max(bytes.len() / 0xFFFF, 1);
            let mut packets = Vec::with_capacity(1 + required_packets);
            packets.push(SilkroadFrame::MassiveHeader {
                count: 0,
                crc: 0,
                contained_opcode: opcode,
                contained_count: required_packets as u16,
            });
            for _ in 0..required_packets {
                packets.push(SilkroadFrame::MassiveContainer {
                    count: 0,
                    crc: 0,
                    inner: bytes.split_to(min(0xFFFF, bytes.len())),
                });
            }
            packets
        } else {
            let encrypted = packet.is_encrypted();
            let (opcode, data) = packet.into_serialize();
            vec![SilkroadFrame::Packet {
                count: 0,
                crc: 0,
                opcode,
                encrypted,
                data,
            }]
        }
    }

    /// Tries to parse the first possible frame from the given data slice.
    /// It will also try to decrypt the frame, if it's encrypted. In
    /// addition to the created frame, it will also return the size of
    /// consumed bytes by the frame.
    pub fn parse(
        data: &[u8],
        security: &Option<Arc<RwLock<SilkroadSecurity>>>,
    ) -> Result<(usize, SilkroadFrame), FrameError> {
        if data.len() < 4 {
            return Err(FrameError::Incomplete);
        }

        let length = LittleEndian::read_u16(&data[0..2]);
        let data = &data[2..];
        let encrypted = length & 0x8000 != 0;
        let content_size = (length & 0x7FFF) as usize;
        let total_size = if encrypted {
            Self::find_next_encrypted_length(content_size + 4)
        } else {
            content_size + 4
        };

        if data.len() < total_size {
            return Err(FrameError::Incomplete);
        }

        let data = &data[0..total_size];

        let data = if encrypted {
            let span = trace_span!("decryption");
            let _enter = span.enter();
            let security = security.as_ref().ok_or(FrameError::MissingSecurity)?;
            let security = security.read().expect("Security RWLock should not get poisoned");
            security.decrypt(data)?
        } else {
            Bytes::copy_from_slice(data)
        };

        let opcode = LittleEndian::read_u16(&data[0..2]);
        let count = data[2];
        let crc = data[3];

        let final_length = total_size + 2;
        if opcode == MASSIVE_PACKET_OPCODE {
            let mode = data[4];
            if mode == 1 {
                // 1 == Header
                let inner_amount = LittleEndian::read_u16(&data[5..7]);
                let inner_opcode = LittleEndian::read_u16(&data[7..9]);
                Ok((
                    final_length,
                    SilkroadFrame::MassiveHeader {
                        count,
                        crc,
                        contained_opcode: inner_opcode,
                        contained_count: inner_amount,
                    },
                ))
            } else {
                Ok((
                    final_length,
                    SilkroadFrame::MassiveContainer {
                        count,
                        crc,
                        inner: Bytes::copy_from_slice(&data[5..]),
                    },
                ))
            }
        } else {
            Ok((
                final_length,
                SilkroadFrame::Packet {
                    count,
                    crc,
                    opcode,
                    encrypted,
                    data: Bytes::copy_from_slice(&data[4..]),
                },
            ))
        }
    }

    /// Computes the size that should be used for the length header field.
    /// Depending on the type of frame this is either:
    /// - The size of the contained data (basic frame)
    /// - A fixed size (massive header frame)
    /// - Container and data size (massive container frame)
    pub fn content_size(&self) -> usize {
        match &self {
            SilkroadFrame::Packet { data, .. } => data.len(),
            SilkroadFrame::MassiveHeader { .. } => {
                // Massive headers have a fixed length because they're always:
                // 1 Byte 'is header', 2 Bytes 'amount of packets', 2 Bytes 'opcode', 1 Byte finish
                6
            },
            SilkroadFrame::MassiveContainer { inner, .. } => {
                // 1 at the start to denote that this is container packet
                // 1 in each content to denote there's more
                1 + inner.len()
            },
        }
    }

    /// Computes the total size of the network packet for this frame.
    /// This is different from [Self::content_size] as it includes
    /// the size of the header as well as the correct size for
    /// encrypted packets.
    pub fn packet_size(&self) -> usize {
        6 + match &self {
            SilkroadFrame::Packet { data, encrypted, .. } => {
                if *encrypted {
                    Self::find_next_encrypted_length(data.len() + 4) - 4
                } else {
                    self.content_size()
                }
            },
            _ => self.content_size(),
        }
    }

    pub fn opcode(&self) -> u16 {
        match &self {
            SilkroadFrame::Packet { opcode, .. } => *opcode,
            _ => 0x600D,
        }
    }

    pub fn serialize(&self, security: &Option<Arc<RwLock<SilkroadSecurity>>>) -> Result<Bytes, FrameError> {
        let mut output = BytesMut::with_capacity(self.packet_size());

        match &self {
            SilkroadFrame::Packet {
                count,
                crc,
                opcode,
                encrypted,
                data,
            } => {
                if *encrypted {
                    let span = trace_span!("encryption");
                    let _guard = span.enter();
                    let mut to_encrypt = BytesMut::with_capacity(data.len() + 4);
                    to_encrypt.put_u16_le(*opcode);
                    to_encrypt.put_u8(*count);
                    to_encrypt.put_u8(*crc);
                    to_encrypt.extend_from_slice(data);
                    let security = security.as_ref().ok_or(FrameError::MissingSecurity)?;
                    let security = security.read().expect("Security RWLock should not get poisoned");
                    let encrypted = security.encrypt(&to_encrypt)?;
                    output.put_u16_le((self.content_size() | 0x8000) as u16);
                    output.put_slice(&encrypted);
                } else {
                    output.put_u16_le(self.content_size() as u16);
                    output.put_u16_le(*opcode);
                    output.put_u8(*count);
                    output.put_u8(*crc);
                    output.put_slice(data);
                }
            },
            SilkroadFrame::MassiveHeader {
                count,
                crc,
                contained_opcode,
                contained_count,
            } => {
                output.put_u16_le(self.content_size() as u16);
                output.put_u16_le(MASSIVE_PACKET_OPCODE);
                output.put_u8(*count);
                output.put_u8(*crc);
                output.put_u8(1);
                output.put_u16_le(*contained_count);
                output.put_u16_le(*contained_opcode);
                output.put_u8(0);
            },
            SilkroadFrame::MassiveContainer { count, crc, inner } => {
                output.put_u16_le(self.content_size() as u16);
                output.put_u16_le(MASSIVE_PACKET_OPCODE);
                output.put_u8(*count);
                output.put_u8(*crc);
                output.put_u8(0);
                output.put_slice(inner);
            },
        }

        Ok(output.freeze())
    }
}
