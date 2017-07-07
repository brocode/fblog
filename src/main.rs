use ansi_term::{Colour, Style};
use clap::{App, AppSettings, Arg};
use serde_json::{Value, Map};
use std::io::{self, BufRead};

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate ansi_term;

fn main() {
  let app = app();
  let matches = app.get_matches();
  let additional_values: Vec<String> = matches.values_of("additional-value")
                                            .map(|values| values.map(|v| v.to_owned()).collect())
                                            .unwrap_or_default();

  let dump_all = matches.is_present("dump-all");

  let stdin = io::stdin();
  let reader = stdin.lock();
  for line in reader.lines() {
    let read_line = &line.expect("Should be able to read line");
    match serde_json::from_str::<Value>(&read_line) {
      Ok(Value::Object(log_entry)) => print_log_line(&log_entry, &additional_values, dump_all),
      _ => println!("??? > {}", read_line),
    };
  }
}

fn app<'a>() -> App<'a, 'a> {
  App::new("fblog")
    .setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(Arg::with_name("additional-value")
           .short("a")
           .multiple(true)
           .takes_value(true)
           .help("adds additional values"))
    .arg(Arg::with_name("dump-all")
           .short("d")
           .multiple(false)
           .takes_value(false)
           .help("dumps all values"))
}

fn get_string_value(value: &Map<String, Value>, keys: &[&str]) -> Option<String> {
  let maybe_match = keys.iter()
                        .fold(None::<&Value>, |maybe_match, key| {
    maybe_match.or(value.get(*key))
  });
  match maybe_match {
    Some(&Value::String(ref level)) => Some(level.to_string()),
    _ => None,
  }
}

fn get_string_value_or_default(value: &Map<String, Value>, keys: &[&str], default: &str) -> String {
  get_string_value(value, keys).unwrap_or(default.to_string())
}

fn level_to_style(level: &str) -> Style {
  match level.to_lowercase().as_ref() {
    "info" => Colour::Green,
    "warn" | "warning" => Colour::Yellow,
    "error" | "err" => Colour::Red,
    "debug" => Colour::Blue,
    _ => Colour::Purple,
  }
  .bold()
}

fn print_log_line(log_entry: &Map<String, Value>, additional_values: &[String], dump_all: bool) {
  let bold = Style::new().bold();

  let level = get_string_value_or_default(log_entry, &["level", "severity"], "unknown");

  let formatted_level = format!("{:>5.5}:", level.to_uppercase());

  let level_style = level_to_style(&level);

  let message = get_string_value_or_default(log_entry, &["short_message", "msg", "message"], "");
  let timestamp = get_string_value_or_default(log_entry, &["timestamp", "time"], "");
  let painted_timestamp = bold.paint(format!("{:>19.19}", timestamp));

  println!("{} {} {}",
           painted_timestamp,
           level_style.paint(formatted_level),
           message);
  if dump_all {
    let all_values: Vec<String> = log_entry.keys().map(|k| k.to_owned()).collect();
    write_additional_values(log_entry, all_values.as_slice());
  } else {
    write_additional_values(log_entry, additional_values);
  }
}

fn write_additional_values(log_entry: &Map<String, Value>, additional_values: &[String]) {
  let bold_grey = Colour::RGB(150, 150, 150).bold();
  for additional_value in additional_values {
    if let Some(value) = get_string_value(log_entry, &[additional_value]) {
      let trimmed_additional_value = format!("{:>25.25}:", additional_value.to_string());
      let painted_value = bold_grey.paint(trimmed_additional_value);
      println!("{} {}", painted_value, value);
    }
  }
}
