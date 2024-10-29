use skrillax_serde::SerializationError;
use std::io;
use std::io::Error;
use std::string::{FromUtf16Error, FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("The opcode '{0}' was not recognized")]
    UnknownOpcode(u16),
    #[error("Could not (de)serialize a packet because of: {0}")]
    SerializationError(#[from] SerializationError),
    #[error("A massive container was received without a massive header previously")]
    StrayMassivePacket,
}

impl From<io::Error> for ProtocolError {
    fn from(io_err: Error) -> Self {
        ProtocolError::SerializationError(SerializationError::from(io_err))
    }
}

impl From<FromUtf8Error> for ProtocolError {
    fn from(utf8_error: FromUtf8Error) -> Self {
        ProtocolError::SerializationError(SerializationError::from(utf8_error))
    }
}

impl From<FromUtf16Error> for ProtocolError {
    fn from(utf16_error: FromUtf16Error) -> Self {
        ProtocolError::SerializationError(SerializationError::from(utf16_error))
    }
}
