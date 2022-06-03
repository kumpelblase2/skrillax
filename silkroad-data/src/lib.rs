pub mod gold;
pub mod level;

use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

pub fn load_lines<T: FromStr>(source: &str) -> Vec<T>
where
    <T as FromStr>::Err: Debug,
{
    source.lines().map(|line| line.parse().unwrap()).collect()
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Column {0} is missing.")]
    MissingColumn(u8),
    #[error("A number could not be parsed: {0}")]
    NumberParseError(#[from] ParseIntError),
}
