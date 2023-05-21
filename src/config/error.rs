use serde::Deserialize;

use std::{fmt::Display, io::Error as IOError};
use toml::de::Error as TomlError;

#[derive(Debug, Deserialize)]
pub enum Error {
  FailedToWrite(String),
  FailedToParse(String),
  FailedToRead(String),
  NoDefault,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
  pub(crate) fn failed_to_write<E: Display>(err: E) -> Self {
    Self::FailedToWrite(err.to_string())
  }
  pub(crate) fn failed_to_read<E: Display>(err: E) -> Self {
    Self::FailedToRead(err.to_string())
  }
}

impl From<TomlError> for Error {
  fn from(inner: TomlError) -> Self {
    Self::FailedToParse(inner.to_string())
  }
}

impl From<toml::ser::Error> for Error {
  fn from(inner: toml::ser::Error) -> Self {
    Self::FailedToWrite(inner.to_string())
  }
}

impl From<IOError> for Error {
  fn from(value: IOError) -> Self {
    Self::FailedToRead(value.to_string())
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::FailedToParse(e) => {
        f.write_str("Failed to parse config: ")?;
        e.fmt(f)
      }
      Self::FailedToWrite(e) => {
        f.write_str("Failed to write config: ")?;
        e.fmt(f)
      }
      Self::FailedToRead(e) => {
        f.write_str("Failed to read config: ")?;
        e.fmt(f)
      }
      Self::NoDefault => f.write_str("Default config file was not found"),
    }
  }
}
