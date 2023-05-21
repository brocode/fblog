use crate::{log::LogSettings, template};

use super::profile::Profile;

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
