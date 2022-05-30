use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use silkroad_protocol::ServerPacket;
use std::cmp::{max, min};
use std::sync::{Arc, RwLock};
use thiserror::Error;
use tracing::trace_span;

use silkroad_security::security::{SilkroadSecurity, SilkroadSecurityError};

const MASSIVE_PACKET_OPCODE: u16 = 0x600D;

pub enum SilkroadFrame {
    Packet {
        count: u8,
        crc: u8,
        opcode: u16,
        encrypted: bool,
        data: Bytes,
    },
    MassiveHeader {
        count: u8,
        crc: u8,
        contained_opcode: u16,
        contained_count: u16,
    },
    MassiveContainer {
        count: u8,
        crc: u8,
        inner: Bytes,
    },
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("I/O error when reading/writing from/to stream")]
    IoError(#[from] std::io::Error),
    #[error("The frame has not been completely transmitted yet")]
    Incomplete,
    #[error("The frame is encrypted, but no security was provided")]
    MissingSecurity,
    #[error("Error when encrypting/decrypting the frame")]
    SecurityError(#[from] SilkroadSecurityError),
    #[error("Expected the packet to be massive")]
    ExpectedMassivePacket,
}

impl SilkroadFrame {
    fn find_next_encrypted_length(length: usize) -> usize {
        SilkroadSecurity::find_encrypted_length(length)
    }

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

        if data.len() < total_size as usize {
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

        let final_length = (total_size + 2) as usize;
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
                    let _enter = span.enter();
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
