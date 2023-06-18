use std::io;
use std::string::{FromUtf16Error, FromUtf8Error};
use thiserror::Error;

/// Any kind of problem that may occur when trying to deserialize data.
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("I/O error when serialize/deserializing packet")]
    IoError(#[from] io::Error),
    #[error("The enum {1} does not have a variation for value {0}")]
    UnknownVariation(usize, &'static str),
    #[error("Could not convert bytes to a string")]
    StringParsingFailed(#[from] FromUtf8Error),
    #[error("Could not convert bytes to a utf16 string")]
    Utf16ParsingFailed(#[from] FromUtf16Error),
}
