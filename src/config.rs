use std::fs;

use serde::{Deserialize, Serialize};

use crate::template::{DEFAULT_ADDITIONAL_VALUE_FORMAT, DEFAULT_MAIN_LINE_FORMAT};

fn default_message_keys() -> Vec<String> {
  vec!["short_message".to_string(), "msg".to_string(), "message".to_string()]
}

fn default_time_keys() -> Vec<String> {
  vec!["timestamp".to_string(), "time".to_string(), "@timestamp".to_string()]
}

fn default_level_keys() -> Vec<String> {
  vec!["level".to_string(), "severity".to_string(), "log.level".to_string(), "loglevel".to_string()]
}

fn default_main_line_format() -> String {
  DEFAULT_MAIN_LINE_FORMAT.to_string()
}

fn default_additional_value_format() -> String {
  DEFAULT_ADDITIONAL_VALUE_FORMAT.to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  #[serde(default = "default_message_keys")]
  pub message_keys: Vec<String>,

  #[serde(default = "default_time_keys")]
  pub time_keys: Vec<String>,

  #[serde(default = "default_level_keys")]
  pub level_keys: Vec<String>,

  #[serde(default = "default_main_line_format")]
  pub main_line_format: String,

  #[serde(default = "default_additional_value_format")]
  pub additional_value_format: String,
}

impl Config {
  pub fn load() -> Option<Config> {
    let mut config_file = dirs::config_dir()?;
    config_file.push("fblog.toml");
    let config_string = fs::read_to_string(config_file).ok()?;
    toml::from_str(&config_string).ok()?
  }
  pub fn new() -> Config {
    Config {
      message_keys: default_message_keys(),
      time_keys: default_time_keys(),
      level_keys: default_level_keys(),
      main_line_format: default_main_line_format(),
      additional_value_format: default_additional_value_format(),
    }
  }

  pub fn get() -> Config {
    Config::load().unwrap_or_else(Config::new)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  #[test]
  fn read_defaults_from_empty_config() {
    let config: Config = toml::from_str(
      r#"
    "#,
    )
    .unwrap();

    assert_eq!(config.level_keys, default_level_keys());
    assert_eq!(config.time_keys, default_time_keys());
    assert_eq!(config.message_keys, default_message_keys());
    assert_eq!(config.main_line_format, DEFAULT_MAIN_LINE_FORMAT);
    assert_eq!(config.additional_value_format, DEFAULT_ADDITIONAL_VALUE_FORMAT);

    let serialized_defaults = toml::to_string(&config).unwrap();
    let default_config_for_documentation = fs::read_to_string("default_config.toml").unwrap();
    assert_eq!(serialized_defaults, default_config_for_documentation)
  }
}
