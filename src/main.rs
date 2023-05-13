use std::io;

#[cfg(test)]
extern crate regex;

mod app;
mod filter;
mod log;
mod message_template;
mod no_color_support;
mod process;
mod template;

use crate::log::LogSettings;
use message_template::MessageTemplate;
use std::fs;

fn main() {
  let app = app::app();
  let matches = app.get_matches();

  let mut log_settings = LogSettings::new_default_settings();

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

  if let Some(context) = matches.get_one::<String>("context-key") {
    log_settings.add_message_template(MessageTemplate::new(context.clone()))
  }

  if let Some(format) = matches.get_one::<String>("message-template-format") {
    if let Err(e) = log_settings.set_message_template_format(format) {
      panic!("Invalid message template format: {}", e)
    }
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

  // TODO: include profile
  let main_line_format = matches
    .get_one::<String>("main-line-format")
    .map(|s| s.to_string())
    .unwrap_or_else(|| template::DEFAULT_MAIN_LINE_FORMAT.to_string());
  let additional_value_format = matches
    .get_one::<String>("additional-value-format")
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
