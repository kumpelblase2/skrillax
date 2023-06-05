pub mod characterdata;
pub mod common;
pub mod datamap;
pub mod entity_rarity;
pub mod gold;
pub mod itemdata;
pub mod level;
pub mod masterydata;
pub mod npc_pos;
pub mod skilldata;
pub mod type_id;

pub use datamap::*;
use encoding_rs::WINDOWS_1252;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use pk2::ChainLookupError;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{FromStr, ParseBoolError};
use thiserror::Error;
pub use type_id::*;

pub(crate) fn load_lines<T: FromStr>(source: &str) -> Result<Vec<T>, T::Err> {
    let mut all_lines = Vec::new();
    for line in source.lines().filter(|line| !line.is_empty()) {
        let parsed = line.parse().map_err(|err| {
            println!("Error with line: {line}");
            err
        })?;
        all_lines.push(parsed);
    }
    Ok(all_lines)
}

pub(crate) fn parse_file<T: FromStr<Err = ParseError>, S: Read>(file: &mut S) -> Result<Vec<T>, FileError> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let (full_string, _, _) = WINDOWS_1252.decode(&buffer);
    Ok(load_lines(full_string.as_ref())?)
}

pub(crate) fn list_files<S: Read>(file: &mut S) -> Result<Vec<String>, FileError> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let (full_string, _, _) = WINDOWS_1252.decode(&buffer);
    Ok(full_string.split("\r\n").map(|s| s.to_string()).collect())
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("Could not parse line in file: {0}")]
    ParseError(#[from] ParseError),
    #[error("I/O level error occurred when reading file: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Column {0} is missing.")]
    MissingColumn(u8),
    #[error("A number could not be parsed: {0}")]
    NumberParseError(#[from] ParseIntError),
    #[error("Could not parse to boolean: {0}")]
    BooleanParseError(#[from] ParseBoolError),
    #[error("A number could not be parsed: {0}")]
    FloatParseError(#[from] ParseFloatError),
    #[error("Unknown variant for enum '{1}': {0}")]
    UnknownVariant(u8, &'static str),
}

impl<T: TryFromPrimitive<Primitive = u8>> From<TryFromPrimitiveError<T>> for ParseError {
    fn from(primitive_error: TryFromPrimitiveError<T>) -> Self {
        ParseError::UnknownVariant(primitive_error.number, T::NAME)
    }
}

impl From<ChainLookupError> for FileError {
    fn from(chain_error: ChainLookupError) -> Self {
        FileError::IoError(chain_error.into())
    }
}
