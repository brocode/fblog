use ansi_term::{Colour, Style};
use serde_json::Value;
use std::io::{self, BufRead};
use clap::{App, Arg, AppSettings};

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate ansi_term;

fn main() {
  let app = App::new("fblog")
    .setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(Arg::with_name("additional-value")
           .short("a")
           .multiple(true)
           .takes_value(true)
           .help("adds additional values"));
  let matches = app.get_matches();
  let additional_values: Vec<&str> = matches.values_of("additional-value").map(|values| values.collect()).unwrap_or_default();

  let stdin = io::stdin();
  let reader = stdin.lock();
  for line in reader.lines() {
    let read_line = &line.expect("Should be able to read line");
    match serde_json::from_str::<Value>(&read_line) {
      Ok(value) => print_log_line(&value, &additional_values),
      Err(_) => println!("??? > {}", read_line),
    };
  }
}


fn get_string_value(value: &Value, keys: &[&str]) -> Option<String> {
  let maybe_match = keys.iter()
                        .fold(None::<&Value>, |maybe_match, key| {
    maybe_match.or(value.get(key))
  });
  match maybe_match {
    Some(&Value::String(ref level)) => Some(level.to_string()),
    _ => None,
  }
}

fn get_string_value_or_default(value: &Value, keys: &[&str], default: &str) -> String {
  get_string_value(value, keys).unwrap_or(default.to_string())
}

fn print_log_line(value: &Value, additional_values: &[&str]) {
  let bold = Style::new().bold();
  let bold_grey = Colour::RGB(150, 150, 150).bold();
  let level = get_string_value_or_default(value, &["level"], "unknown");

  let formatted_level = format!("{:>7.7}", level);

  let colour = match level.to_lowercase().as_ref() {
    "info" => Colour::Green,
    "warn" => Colour::Yellow,
    "warning" => Colour::Yellow,
    "error" => Colour::Red,
    "debug" => Colour::Blue,
    _ => Colour::Purple,
  }.bold();

  let message = get_string_value_or_default(value, &["short_message", "msg"], "");
  let timestamp = get_string_value_or_default(value, &["timestamp", "time"], "");
  let painted_timestamp = bold.paint(format!("{:>19.19}", timestamp));

  println!("{} {} - {}",
           painted_timestamp,
           colour.paint(formatted_level),
           message);
  for additional_value in additional_values {
    if let Some(value) = get_string_value(value, &[additional_value]){
      let trimmed_additional_value = format!("@{:<10.10}", additional_value.to_string());
      let painted_value = bold_grey.paint(trimmed_additional_value);
      println!("                {}   {}", painted_value, value);
    }
  }
}
