use std::io;

#[cfg(test)]
extern crate regex;

mod app;
mod filter;
mod inspect;
mod log;
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
  log_settings.inspect = matches.is_present("inspect");

  let implicit_return = !matches.is_present("no-implicit-filter-return-statement");
  let maybe_filter = matches.value_of("filter");

  let input_filename = matches.value_of("INPUT").unwrap();
  let mut input = io::BufReader::new(input_read(input_filename));

  process::process_input(&log_settings, &mut input, maybe_filter, implicit_return)
}

fn input_read(input_filename: &str) -> Box<dyn io::Read> {
  if input_filename == "-" {
    Box::new(io::stdin())
  } else {
    Box::new(fs::File::open(input_filename).unwrap_or_else(|_| panic!("Can't open file: {}", input_filename)))
  }
}
