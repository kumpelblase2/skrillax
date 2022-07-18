pub mod characterdata;
pub mod gold;
pub mod level;

use encoding_rs::WINDOWS_1252;
use pk2::ChainLookupError;
use std::fmt::Debug;
use std::io;
use std::io::Read;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

pub(crate) fn load_lines<T: FromStr>(source: &str) -> Result<Vec<T>, T::Err> {
    let mut all_lines = Vec::new();
    for line in source.lines().filter(|line| line.len() > 0) {
        let parsed = line.parse()?;
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
}

impl From<ChainLookupError> for FileError {
    fn from(chain_error: ChainLookupError) -> Self {
        FileError::IoError(chain_error.into())
    }
}
