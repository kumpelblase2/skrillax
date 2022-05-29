use std::io;
use std::string::{FromUtf16Error, FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("I/O error when serialize/deserializing packet")]
    IoError(#[from] io::Error),
    #[error("The opcode '{0}' was not recognized")]
    UnknownOpcode(u16),
    #[error("The enum {1} does not have a variation for value {0}")]
    UnknownVariation(u8, &'static str),
    #[error("Could not convert bytes to a string")]
    StringParsingFailed(#[from] FromUtf8Error),
    #[error("Could not convert bytes to a utf16 string")]
    Utf16ParsingFailed(#[from] FromUtf16Error),
    #[error("A massive container was received without a massive header previously")]
    StrayMassivePacket,
}
