use clap::parser::ValueSource;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml_edit::{value, Array, Document};

use super::error::{Error, Result};
use crate::{log::LogSettings, template};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Profile {
  #[serde(flatten)]
  pub log_settings: LogSettings,

  #[serde(flatten)]
  pub template_settings: template::Settings,
}

impl Profile {
  fn profile_path(profile: &str) -> PathBuf {
    super::config_dir().join("profiles").join(PathBuf::from(profile).with_extension("toml"))
  }

  pub fn load(profile: &str) -> Result<Profile> {
    let profile_str = fs::read_to_string(Self::profile_path(profile))?;

    toml::from_str(&profile_str)?
  }

  pub fn update_from_matches(profile: &str, matches: &clap::ArgMatches) -> Result<()> {
    let profile_file = Self::profile_path(profile);
    let profile_str = fs::read_to_string(&profile_file).unwrap_or_default();
    let mut profile = profile_str.parse::<Document>().expect("invalid profile");

    if let Some(values) = matches.get_many::<String>("additional-value") {
      profile["additional_values"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(values) = matches.get_many::<String>("message-key") {
      profile["message_keys"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(values) = matches.get_many::<String>("time-key") {
      profile["time_keys"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(values) = matches.get_many::<String>("level-key") {
      profile["level_keys"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(values) = matches.get_many::<String>("context-key") {
      profile["substitution_enabled"] = value(true); // Substition is implicitly enabled by context keys
      profile["context_keys"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(placeholder_format) = matches.get_one::<String>("placeholder-format") {
      profile["substitution_enabled"] = value(true); // Substition is implicitly enabled by placeholder format
      profile["placeholder_format"] = value(placeholder_format);
    }

    if let Some(ValueSource::CommandLine) = matches.value_source("dump-all") {
      profile["dump_all"] = value(matches.get_flag("dump-all"));
    }
    if let Some(ValueSource::CommandLine) = matches.value_source("with-prefix") {
      profile["with_prefix"] = value(matches.get_flag("with-prefix"));
    }
    if let Some(ValueSource::CommandLine) = matches.value_source("print-lua") {
      profile["print_lua"] = value(matches.get_flag("print-lua"));
    }

    if let Some(values) = matches.get_many::<String>("excluded-value") {
      profile["dump_all"] = value(true); // Dump all is implicitly set by exclusion
      profile["excluded_values"] = value(Array::from_iter(values.cloned()));
    }

    if let Some(ValueSource::CommandLine) = matches.value_source("main-line-format") {
      let main_line_format = matches.get_one::<String>("main-line-format").unwrap();
      profile["main_line_format"] = value(main_line_format);
    }

    if let Some(ValueSource::CommandLine) = matches.value_source("additional-value-format") {
      let additional_value_format = matches.get_one::<String>("additional-value-format").unwrap();
      profile["additional_value_format"] = value(additional_value_format);
    }

    if let Some(parent) = profile_file.parent() {
      fs::create_dir_all(parent).map_err(Error::failed_to_write)?;
    }

    fs::write(profile_file, profile.to_string()).map_err(Error::failed_to_write)
  }
}
