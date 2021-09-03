use std::io;

#[cfg(test)]
extern crate regex;

mod app;
mod filter;
mod log;
mod no_color_support;
mod process;
mod template;

use crate::log::LogSettings;
use std::fs;

fn main() {
  let app = app::app();
  let matches = app.get_matches();

  let mut log_settings = LogSettings::new_default_settings();

  if let Some(values) = matches.values_of("additional-value") {
    log_settings.add_additional_values(values.map(ToString::to_string).collect());
  }

  if let Some(values) = matches.values_of("message-key") {
    log_settings.add_message_keys(values.map(ToString::to_string).collect());
  }

  if let Some(values) = matches.values_of("time-key") {
    log_settings.add_time_keys(values.map(ToString::to_string).collect());
  }

  if let Some(values) = matches.values_of("level-key") {
    log_settings.add_level_keys(values.map(ToString::to_string).collect());
  }

  log_settings.dump_all = matches.is_present("dump-all");
  log_settings.with_prefix = matches.is_present("with-prefix");

  let implicit_return = !matches.is_present("no-implicit-filter-return-statement");
  let maybe_filter = matches.value_of("filter");

  let input_filename = matches.value_of("INPUT").unwrap();
  let mut input = io::BufReader::new(input_read(input_filename));

  // TODO: include profile
  let main_line_format = matches
    .value_of("main-line-format")
    .map(|s| s.to_string())
    .unwrap_or_else(|| template::DEFAULT_MAIN_LINE_FORMAT.to_string());
  let additional_value_format = matches
    .value_of("additional-value-format")
    .map(|s| s.to_string())
    .unwrap_or_else(|| template::DEFAULT_ADDITIONAL_VALUE_FORMAT.to_string());

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
