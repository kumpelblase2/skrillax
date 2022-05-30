use crate::frame::{FrameError, SilkroadFrame};
use bytes::{Buf, BytesMut};
use silkroad_security::security::SilkroadSecurity;
use std::sync::{Arc, RwLock};
use tokio_util::codec::{Decoder, Encoder};
use tracing::debug;

pub struct SilkroadFrameEncoder {
    security: Option<Arc<RwLock<SilkroadSecurity>>>,
}

impl SilkroadFrameEncoder {
    pub fn new(security: Option<Arc<RwLock<SilkroadSecurity>>>) -> Self {
        SilkroadFrameEncoder { security }
    }
}

impl Encoder<SilkroadFrame> for SilkroadFrameEncoder {
    type Error = FrameError;

    fn encode(&mut self, item: SilkroadFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        debug!("Sending packet with opcode {:#04X}", item.opcode());
        let bytes = item.serialize(&self.security)?;
        dst.extend_from_slice(&bytes);
        Ok(())
    }
}

pub struct SilkroadFrameDecoder {
    security: Option<Arc<RwLock<SilkroadSecurity>>>,
}

impl SilkroadFrameDecoder {
    pub fn new(security: Option<Arc<RwLock<SilkroadSecurity>>>) -> Self {
        SilkroadFrameDecoder { security }
    }
}

impl Decoder for SilkroadFrameDecoder {
    type Item = SilkroadFrame;
    type Error = FrameError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match SilkroadFrame::parse(src, &self.security) {
            Ok((bytes_read, frame)) => {
                debug!("Received packet for opcode {:#04X}", frame.opcode());
                src.advance(bytes_read);
                Ok(Some(frame))
            },
            Err(FrameError::Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
