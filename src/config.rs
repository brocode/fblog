use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fmt::Display,
  fs,
  io::Error as IOError,
  path::{Path, PathBuf},
};
use toml::de::Error as TomlError;

use crate::{log::LogSettings, template};

#[derive(Debug, Deserialize)]
pub enum Error {
  FailedToWrite(String),
  FailedToParse(String),
  FailedToRead(String),
  NoDefault,
}

impl Error {
  fn failed_to_write<E: Display>(err: E) -> Self {
    Self::FailedToWrite(err.to_string())
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

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
  pub default_profile: Option<String>,
  #[serde(rename = "profile")]
  pub profiles: HashMap<String, Profile>,
}

impl Config {
  pub fn config_file_path() -> PathBuf {
    ProjectDirs::from("org", "brocode", "fblog")
      .expect("OS home directory")
      .config_local_dir()
      .join("config.toml")
  }

  pub fn load_default() -> Result<Config, Error> {
    let config_file = Self::config_file_path();
    if !config_file.exists() {
      return Ok(Config::default())
    }
    Self::load_from_file(&config_file)
  }

  fn load_from_file(config_file: &Path) -> Result<Config, Error> {
    let config_str = fs::read_to_string(config_file)?;

    toml::from_str(&config_str)?
  }

  pub fn get_default_profile(&self) -> Profile {
    self.profiles.get("default").cloned().unwrap_or_default()
  }

  pub fn save_default_profile(profile: &str) -> Result<(), Error> {
    let mut config = Self::load_default().unwrap_or_default();
    config.default_profile = Some(profile.to_owned());
    let config_file = Self::config_file_path();
    if let Some(parent) = config_file.parent() {
      fs::create_dir_all(parent).map_err(Error::failed_to_write)?;
    }
    let config_str = toml::to_string_pretty(&config)?;
    fs::write(config_file, config_str).map_err(Error::failed_to_write)
  }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Profile {
  #[serde(flatten)]
  pub log_settings: LogSettings,

  #[serde(flatten)]
  pub template_settings: template::Settings,

  #[serde(skip)]
  pub maybe_filter: Option<String>,

  #[serde(skip)]
  pub implicit_return: bool,
}

pub struct Options {
  pub log_settings: LogSettings,
  pub template_settings: template::Settings,
  pub maybe_filter: Option<String>,
  pub implicit_return: bool,
}

impl From<Profile> for Options {
  fn from(value: Profile) -> Self {
    Self {
      implicit_return: true,
      maybe_filter: None,
      log_settings: value.log_settings,
      template_settings: value.template_settings,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_serialize() {
    let config: Config = toml::from_str(
      r#"
            default_profile = 'foo'

            [profile.foo]
            message_keys = ['msg']

            [profile.bar]
            message_keys = ['data']
        "#,
    )
    .expect("input config should parse");

    assert_eq!(config.default_profile, Some("foo".to_owned()));
    assert!(config.profiles.contains_key("foo"));
    assert!(config.profiles.contains_key("bar"));
    assert_eq!(config.profiles.get("foo").map(|p| &p.log_settings.message_keys), Some(&vec!["msg".to_owned()]));
    assert_eq!(config.profiles.get("bar").map(|p| &p.log_settings.message_keys), Some(&vec!["data".to_owned()]));
  }
}
