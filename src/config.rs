use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
  fs,
  path::{Path, PathBuf},
};
use toml_edit::{value, Document};

use self::error::{Error, Result};
pub use self::options::Options;
use self::profile::Profile;

mod error;
pub mod options;
pub mod profile;

fn config_dir() -> PathBuf {
  ProjectDirs::from("org", "brocode", "fblog")
    .expect("OS home directory")
    .config_local_dir()
    .to_path_buf()
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
  pub default_profile: Option<String>,
  #[serde(flatten, default)]
  pub profile: Option<Profile>,
}

impl Config {
  pub fn config_file_path() -> PathBuf {
    config_dir().join("config.toml")
  }

  pub fn load_default() -> Result<Config> {
    let config_file = Self::config_file_path();
    if !config_file.exists() {
      return Ok(Config::default());
    }
    Self::load_from_file(&config_file)
  }

  fn load_from_file(config_file: &Path) -> Result<Config> {
    let config_str = fs::read_to_string(config_file)?;

    toml::from_str(&config_str).map_err(Error::failed_to_read)
  }

  pub fn get_default_profile(&self) -> Profile {
    self.profile.clone().unwrap_or_default()
  }

  pub fn get_profile(&self, profile: Option<&str>) -> Result<Profile> {
    match profile {
      Some("default") | None => Ok(self.get_default_profile()),
      Some(profile) => Profile::load(profile),
    }
  }

  pub fn save_default_profile(profile: &str) -> Result<()> {
    let config_file = Self::config_file_path();
    let config_str = fs::read_to_string(&config_file).unwrap_or_default();
    let mut config = config_str.parse::<Document>().expect("invalid configuration");
    config["default_profile"] = value(profile);
    if let Some(parent) = config_file.parent() {
      fs::create_dir_all(parent).map_err(Error::failed_to_write)?;
    }
    fs::write(config_file, config.to_string()).map_err(Error::failed_to_write)
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

            message_keys = ['msg']
        "#,
    )
    .expect("input config should parse");

    assert_eq!(config.default_profile, Some("foo".to_owned()));
    assert_eq!(config.profile.map(|p| p.log_settings.message_keys), Some(vec!["msg".to_owned()]));
    // assert_eq!(config.profiles.get("bar").map(|p| &p.log_settings.message_keys), Some(&vec!["data".to_owned()]));
  }
}
