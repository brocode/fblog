use crate::substitution::Substitution;
use crate::{filter, config};
use crate::log::{self, LogSettings};
use crate::no_color_support::style;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde_json::{Map, Value};
use std::io::Write;
use std::io::{self, BufRead};
use yansi::{Color, Style};

lazy_static! {
  static ref BOLD_ORANGE: Style = Color::RGB(255, 135, 22).style().bold();
}

pub fn process_input(
  options: &config::Options,
  input: &mut dyn io::BufRead,
  handlebars: &Handlebars<'static>,
  substitution: Option<&Substitution>
) {
  for line in input.lines() {
    let read_line = &line.expect("Should be able to read line");
    match process_input_line(options, read_line, None, handlebars, substitution) {
      Ok(_) => (),
      Err(_) => print_unknown_line(read_line),
    }
  }
}

fn print_unknown_line(line: &str) {
  let write_result = writeln!(&mut io::stdout(), "{} {}", style(&BOLD_ORANGE, "??? >"), line);
  if write_result.is_err() {
    // Output end reached
    std::process::exit(14);
  }
}

fn process_input_line(
  options: &config::Options,
  read_line: &str,
  maybe_prefix: Option<&str>,
  handlebars: &Handlebars<'static>,
  substitution: Option<&Substitution>,
) -> Result<(), ()> {
  match serde_json::from_str::<Value>(read_line) {
    Ok(Value::Object(log_entry)) => {
      process_json_log_entry(options, maybe_prefix, &log_entry, handlebars, substitution);
      Ok(())
    }
    _ => {
      if options.log_settings.with_prefix && maybe_prefix.is_none() {
        match read_line.find('{') {
          Some(pos) => {
            let prefix = &read_line[..pos];
            let rest = &read_line[pos..];
            process_input_line(options, rest, Some(prefix), handlebars, substitution)
          }
          None => Err(()),
        }
      } else {
        Err(())
      }
    }
  }
}

fn process_json_log_entry(
  options: &config::Options,
  maybe_prefix: Option<&str>,
  log_entry: &Map<String, Value>,
  handlebars: &Handlebars<'static>,
  substitution: Option<&Substitution>,
) {
  if let Some(filter) = &options.maybe_filter {
    match filter::show_log_entry(log_entry, filter, options.implicit_return, &options.log_settings) {
      Ok(true) => process_log_entry(&options.log_settings, maybe_prefix, log_entry, handlebars, substitution),
      Ok(false) => (),
      Err(e) => {
        writeln!(io::stderr(), "{}: '{:?}'", Color::Red.paint("Failed to apply filter expression"), e).expect("Should be able to write to stderr");
      }
    }
  } else {
    process_log_entry(&options.log_settings, maybe_prefix, log_entry, handlebars, substitution)
  }
}

fn process_log_entry(log_settings: &LogSettings, maybe_prefix: Option<&str>, log_entry: &Map<String, Value>, handlebars: &Handlebars<'static>, substitution: Option<&Substitution>) {
  log::print_log_line(&mut io::stdout(), maybe_prefix, log_entry, log_settings, handlebars, substitution)
}
