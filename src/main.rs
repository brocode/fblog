use std::io;

#[cfg(test)]
extern crate regex;

mod app;
mod config;
mod filter;
mod log;
mod log_settings;
mod process;
mod substitution;
mod template;
mod time;

use crate::log_settings::LogSettings;
use clap_complete::{Shell, generate};
use config::Config;
use std::fs;
use substitution::Substitution;

fn main() {
	let app = app::app();
	let matches = app.get_matches();

	if let Some(generator) = matches.get_one::<Shell>("generate-completions").copied() {
		let mut app = app::app();
		let name = app.get_name().to_string();
		eprintln!("Generating completion file for {generator}...");
		generate(generator, &mut app, name, &mut io::stdout());
		return;
	}

	let config: Config = if let Some(config_file_path) = matches.get_one::<String>("config-file") {
		Config::load_from_file(config_file_path).expect("Could not load config file.")
	} else {
		Config::get()
	};

	let mut log_settings = LogSettings::from_config(&config);

	if let Some(values) = matches.get_many::<String>("additional-value") {
		log_settings.add_additional_values(values.map(ToOwned::to_owned).collect());
	}

	if let Some(values) = matches.get_many::<String>("message-key") {
		log_settings.add_message_keys(values.map(ToString::to_string).collect());
	}

	if let Some(values) = matches.get_many::<String>("time-key") {
		log_settings.add_time_keys(values.map(ToString::to_string).collect());
	}

	if let Some(values) = matches.get_many::<String>("level-key") {
		log_settings.add_level_keys(values.map(ToString::to_string).collect());
	}

	if let Some(values) = matches.get_many::<(String, String)>("map-level") {
		log_settings.add_level_map(values.map(ToOwned::to_owned).collect());
	}

	match (matches.get_one::<String>("context-key"), matches.get_one::<String>("placeholder-format")) {
		(None, None) => {
			// Neither context key nor placeholder is set, meaning that substitution is not enabled
			// since setting the flag sets the defaults for those arguments
		}
		(context, format) => match Substitution::new(context, format) {
			Err(e) => panic!("Invalid placeholder format: {}", e),
			Ok(subst) => log_settings.add_substitution(subst),
		},
	}

	log_settings.dump_all = matches.get_flag("dump-all");
	log_settings.with_prefix = matches.get_flag("with-prefix");
	log_settings.print_lua = matches.get_flag("print-lua");

	if let Some(values) = matches.get_many::<String>("excluded-value") {
		log_settings.dump_all = true; // Dump all is implicitly set by exclusion
		log_settings.add_excluded_values(values.map(ToString::to_string).collect());
	}

	let implicit_return = !matches.get_flag("no-implicit-filter-return-statement");
	let maybe_filter = matches.get_one::<String>("filter");

	let input_filename = matches.get_one::<String>("INPUT").unwrap();
	let mut input = io::BufReader::new(input_read(input_filename));

	let main_line_format = matches
		.get_one::<String>("main-line-format")
		.map(|s| s.to_string())
		.unwrap_or_else(|| config.main_line_format.to_string());
	let additional_value_format = matches
		.get_one::<String>("additional-value-format")
		.map(|s| s.to_string())
		.unwrap_or_else(|| config.additional_value_format.to_string());

	let handlebars = template::fblog_handlebar_registry(main_line_format, additional_value_format);
	process::process_input(&log_settings, &mut input, maybe_filter, implicit_return, &handlebars)
}

fn input_read(input_filename: &str) -> Box<dyn io::Read> {
	if input_filename == "-" {
		Box::new(io::stdin())
	} else {
		Box::new(fs::File::open(input_filename).unwrap_or_else(|_| panic!("Can't open file: {}", input_filename)))
	}
}
