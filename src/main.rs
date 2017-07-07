use ansi_term::Colour;
use serde_json::Value;
use std::io::{self, BufRead};

extern crate serde_json;
extern crate ansi_term;

fn main() {
  let stdin = io::stdin();
  let reader = stdin.lock();
  for line in reader.lines() {
    let read_line = &line.expect("Should be able to read line");
    match serde_json::from_str::<Value>(&read_line) {
      Ok(value) => print_log_line(&value),
      Err(_) => println!("??? > {}", read_line),
    };
  }
}


fn get_string_value_or_default(value: &Value, keys: &[&str], default: &str) -> String {
  let maybe_match = keys.iter()
                        .fold(None::<&Value>, |maybe_match, key| {
    maybe_match.or(value.get(key))
  });
  match maybe_match {
    Some(&Value::String(ref level)) => level.to_string(),
    _ => default.to_string(),
  }
}

fn print_log_line(value: &Value) {
  let level = get_string_value_or_default(value, &["level"], "unknown");

  let formatted_level = format!("{:>7.7}", level);

  let colour = match level.to_lowercase().as_ref() {
    "info" => Colour::Green,
    "warn" => Colour::Yellow,
    "warning" => Colour::Yellow,
    "error" => Colour::Red,
    "debug" => Colour::Blue,
    _ => Colour::Purple,
  };

  let message = get_string_value_or_default(value, &["short_message", "msg"], "");
  let timestamp = get_string_value_or_default(value, &["timestamp", "time"], "");

  println!("{:>19.19} {} | {}",
           timestamp,
           colour.paint(formatted_level),
           message)
}
