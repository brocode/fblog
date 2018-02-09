
use ansi_term::Colour;
use clap::{App, AppSettings, Arg};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead};
use std::io::Write;
use std::fs;

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate ansi_term;

extern crate hlua;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
extern crate regex;

mod log;
mod filter;

fn main() {
  let app = app();
  let matches = app.get_matches();
  let bold_orange = Colour::RGB(255, 135, 22).bold();

  let mut log_settings = log::LogSettings::new_default_settings();

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
  let implicit_return = !matches.is_present("no-implicit-filter-return-statement");
  let maybe_filter = matches.value_of("filter");

  let input_filename = matches.value_of("INPUT").unwrap();
  let input: Box<io::Read> = if input_filename == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(
            fs::File::open(input_filename).expect(&format!("Can't open file: {}", input_filename)),
        )
    };
let input = io::BufReader::new(input);
  
  for line in input.lines() {
    let read_line = &line.expect("Should be able to read line");
    match serde_json::from_str::<Value>(read_line) {
      Ok(Value::Object(log_entry)) => {
        let string_log_entry = &extract_string_values(&log_entry);
        if let Some(filter) = maybe_filter {
          match filter::show_log_entry(string_log_entry, filter, implicit_return) {
            Ok(true) => log::print_log_line(&mut io::stdout(), string_log_entry, &log_settings),
            Ok(false) => (),
            Err(e) => {
              writeln!(io::stderr(),
                       "{}: '{:?}'",
                       Colour::Red.paint("Failed to apply filter expression"),
                       e)
                .expect("Should be able to write to stderr");
              std::process::exit(1)
            }
          }
        } else {
          log::print_log_line(&mut io::stdout(), string_log_entry, &log_settings)
        }
      }
      _ => println!("{} {}", bold_orange.paint("??? >"), read_line),
    };
  }
}

fn app<'a>() -> App<'a, 'a> {
  App::new("fblog")
    .global_setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(Arg::with_name("additional-value")
           .long("additional-value")
           .short("a")
           .multiple(true)
           .takes_value(true)
           .help("adds additional values"))
    .arg(Arg::with_name("message-key")
           .long("message-key")
           .short("m")
           .multiple(true)
           .takes_value(true)
           .help("Adds an additional key to detect the message in the log entry."))
    .arg(Arg::with_name("time-key")
           .long("time-key")
           .short("t")
           .multiple(true)
           .takes_value(true)
           .help("Adds an additional key to detect the time in the log entry."))
    .arg(Arg::with_name("level-key")
           .long("level-key")
           .short("l")
           .multiple(true)
           .takes_value(true)
           .help("Adds an additional key to detect the level in the log entry."))
    .arg(Arg::with_name("dump-all")
           .long("dump-all")
           .short("d")
           .multiple(false)
           .takes_value(false)
           .help("dumps all values"))
    .arg(Arg::with_name("filter")
           .long("filter")
           .short("f")
           .multiple(false)
           .takes_value(true)
           .help("lua expression to filter log entries. `message ~= nil and string.find(message, \"text.*\") ~= nil`"))
    .arg(Arg::with_name("no-implicit-filter-return-statement")
           .long("no-implicit-filter-return-statement")
           .multiple(false)
           .takes_value(false)
           .help("if you pass a filter expression 'return' is automatically prepended. Pass this switch to disable the implicit return."))
        .arg(Arg::with_name("INPUT")
          .help("Sets the input file to use, otherwise assumes stdin")
          .required(false)
          .default_value("-")
          .index(1))
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
