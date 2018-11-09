use ansi_term::Colour;
use clap::{App, AppSettings, Arg};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::io::{self, BufRead};

extern crate clap;

extern crate ansi_term;
extern crate serde_json;

extern crate hlua;

#[cfg(test)]
extern crate maplit;

#[cfg(test)]
extern crate regex;

use clap::crate_version;

mod filter;
mod inspect;
mod log;

use inspect::InspectLogger;
use log::LogSettings;

fn main() {
  let app = app();
  let matches = app.get_matches();

  let mut log_settings = LogSettings::new_default_settings();

  if let Some(values) = matches.values_of("additional-value") {
    log_settings.add_additional_values(values.map(|v| v.to_string()).collect());
  }

  if let Some(values) = matches.values_of("message-key") {
    log_settings.add_message_keys(values.map(|v| v.to_string()).collect());
  }

  if let Some(values) = matches.values_of("time-key") {
    log_settings.add_time_keys(values.map(|v| v.to_string()).collect());
  }

  if let Some(values) = matches.values_of("level-key") {
    log_settings.add_level_keys(values.map(|v| v.to_string()).collect());
  }

  log_settings.dump_all = matches.is_present("dump-all");
  log_settings.with_prefix = matches.is_present("prefix");
  log_settings.inspect = matches.is_present("inspect");

  let implicit_return = !matches.is_present("no-implicit-filter-return-statement");
  let maybe_filter = matches.value_of("filter");

  let input_filename = matches.value_of("INPUT").unwrap();
  let mut input = io::BufReader::new(input_read(input_filename));

  process_input(&log_settings, &mut input, maybe_filter, implicit_return)
}

fn input_read(input_filename: &str) -> Box<io::Read> {
  if input_filename == "-" {
    Box::new(io::stdin())
  } else {
    Box::new(fs::File::open(input_filename).unwrap_or_else(|_| panic!("Can't open file: {}", input_filename)))
  }
}

fn print_unknown_line(line: &str) {
  let bold_orange = Colour::RGB(255, 135, 22).bold();
  println!("{} {}", bold_orange.paint("??? >"), line);
}

fn process_input_line(log_settings: &LogSettings, read_line: &str, maybe_prefix: Option<&str>, maybe_filter: Option<&str>, implicit_return: bool) {
  let mut inspect_logger = InspectLogger::new();
  match serde_json::from_str::<Value>(read_line) {
    Ok(Value::Object(log_entry)) => process_json_log_entry(log_settings, &mut inspect_logger, maybe_prefix, &log_entry, maybe_filter, implicit_return),
    _ => {
      if !log_settings.inspect {
        if log_settings.with_prefix && maybe_prefix.is_none() {
          match read_line.find("{") {
            Some(pos) => {
              let prefix = &read_line[..pos];
              let rest = &read_line[pos..];
              process_input_line(log_settings, rest, Some(prefix), maybe_filter, implicit_return);
            }
            None => print_unknown_line(read_line),
          }
        } else {
          print_unknown_line(read_line);
        }
      }
    }
  }
}

fn process_input(log_settings: &LogSettings, input: &mut io::BufRead, maybe_filter: Option<&str>, implicit_return: bool) {
  for line in input.lines() {
    let read_line = &line.expect("Should be able to read line");
    process_input_line(log_settings, read_line, None, maybe_filter, implicit_return);
  }
}

fn process_json_log_entry(
  log_settings: &LogSettings,
  inspect_logger: &mut InspectLogger,
  maybe_prefix: Option<&str>,
  log_entry: &Map<String, Value>,
  maybe_filter: Option<&str>,
  implicit_return: bool,
) {
  let string_log_entry = &extract_string_values(log_entry);
  if let Some(filter) = maybe_filter {
    match filter::show_log_entry(string_log_entry, filter, implicit_return) {
      Ok(true) => process_log_entry(log_settings, inspect_logger, maybe_prefix, string_log_entry),
      Ok(false) => (),
      Err(e) => {
        writeln!(io::stderr(), "{}: '{:?}'", Colour::Red.paint("Failed to apply filter expression"), e).expect("Should be able to write to stderr");
        std::process::exit(1)
      }
    }
  } else {
    process_log_entry(log_settings, inspect_logger, maybe_prefix, string_log_entry)
  }
}

fn process_log_entry(log_settings: &LogSettings, inspect_logger: &mut InspectLogger, maybe_prefix: Option<&str>, log_entry: &BTreeMap<String, String>) {
  if log_settings.inspect {
    inspect_logger.print_unknown_keys(log_entry, &mut io::stdout())
  } else {
    log::print_log_line(&mut io::stdout(), maybe_prefix, log_entry, log_settings)
  }
}

fn app<'a>() -> App<'a, 'a> {
  App::new("fblog")
    .global_setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(
      Arg::with_name("additional-value")
        .long("additional-value")
        .short("a")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("adds additional values"),
    ).arg(
      Arg::with_name("message-key")
        .long("message-key")
        .short("m")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the message in the log entry."),
    ).arg(
      Arg::with_name("time-key")
        .long("time-key")
        .short("t")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the time in the log entry."),
    ).arg(
      Arg::with_name("level-key")
        .long("level-key")
        .short("l")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the level in the log entry."),
    ).arg(
      Arg::with_name("dump-all")
        .long("dump-all")
        .short("d")
        .multiple(false)
        .takes_value(false)
        .help("dumps all values"),
    ).arg(
      Arg::with_name("prefix")
        .long("prefix")
        .short("p")
        .multiple(false)
        .takes_value(false)
        .help("consider all text before opening curly brace as prefix"),
    ).arg(
      Arg::with_name("filter")
        .long("filter")
        .short("f")
        .multiple(false)
        .takes_value(true)
        .help("lua expression to filter log entries. `message ~= nil and string.find(message, \"text.*\") ~= nil`"),
    ).arg(
      Arg::with_name("no-implicit-filter-return-statement")
        .long("no-implicit-filter-return-statement")
        .multiple(false)
        .takes_value(false)
        .help("if you pass a filter expression 'return' is automatically prepended. Pass this switch to disable the implicit return."),
    ).arg(
      Arg::with_name("INPUT")
        .help("Sets the input file to use, otherwise assumes stdin")
        .required(false)
        .default_value("-"),
    ).arg(
      Arg::with_name("inspect")
        .long("inspect")
        .short("i")
        .multiple(false)
        .takes_value(false)
        .help("only prints json keys not encountered before"),
    )
}

fn extract_string_values(log_entry: &Map<String, Value>) -> BTreeMap<String, String> {
  let mut string_map: BTreeMap<String, String> = BTreeMap::new();
  for (key, value) in log_entry {
    if let Value::String(ref string_value) = *value {
      string_map.insert(key.to_owned(), string_value.to_owned());
    }
  }
  string_map
}
