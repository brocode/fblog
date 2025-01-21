use crate::{config::Config, substitution::Substitution};
use std::collections::BTreeMap;

pub struct LogSettings {
	pub message_keys: Vec<String>,
	pub time_keys: Vec<String>,
	pub level_keys: Vec<String>,
	pub level_map: BTreeMap<String, String>,
	pub additional_values: Vec<String>,
	pub excluded_values: Vec<String>,
	pub dump_all: bool,
	pub with_prefix: bool,
	pub print_lua: bool,
	pub substitution: Option<Substitution>,
}

impl LogSettings {
	pub fn from_config(config: &Config) -> LogSettings {
		LogSettings {
			message_keys: config.message_keys.clone(),
			time_keys: config.time_keys.clone(),
			level_keys: config.level_keys.clone(),
			level_map: config.level_map.clone(),
			additional_values: config.always_print_fields.clone(),
			excluded_values: config.dump_all_exclude.clone(),
			dump_all: false,
			with_prefix: false,
			print_lua: false,
			substitution: None,
		}
	}

	#[allow(dead_code)]
	pub fn new_default_settings() -> LogSettings {
		let default_config = Config::new();
		LogSettings::from_config(&default_config)
	}

	pub fn add_additional_values(&mut self, mut additional_values: Vec<String>) {
		self.additional_values.append(&mut additional_values);
	}

	pub fn add_message_keys(&mut self, mut message_keys: Vec<String>) {
		message_keys.append(&mut self.message_keys);
		self.message_keys = message_keys;
	}

	pub fn add_time_keys(&mut self, mut time_keys: Vec<String>) {
		time_keys.append(&mut self.time_keys);
		self.time_keys = time_keys;
	}

	pub fn add_level_keys(&mut self, mut level_keys: Vec<String>) {
		level_keys.append(&mut self.level_keys);
		self.level_keys = level_keys;
	}

	pub fn add_level_map(&mut self, values: Vec<(String, String)>) {
		self.level_map.extend(values);
	}

	pub fn add_excluded_values(&mut self, mut excluded_values: Vec<String>) {
		self.excluded_values.append(&mut excluded_values);
	}

	pub fn add_substitution(&mut self, message_template: Substitution) {
		self.substitution = Some(message_template)
	}
}
