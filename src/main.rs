use std::io::{self, BufRead};
use serde_json::Value;
use ansi_term::Colour;

extern crate serde_json;
extern crate ansi_term;

fn main() {
   let stdin = io::stdin();
  let reader = stdin.lock();
  for line in reader.lines() {
    match serde_json::from_str::<Value>(&line.expect("Should be able to read line")) {
      Ok(value) => print_log_line(&value),
      Err(_) => println!("fb: Unparseable line")
    };
  }
}


fn get_string_value_or_default(value: &Value, key: &str, default: &str) -> String {
  match value.get(key) {
    Some(&Value::String(ref level)) => level.to_string(),
    _ => default.to_string()
  }
}

fn print_log_line(value: &Value) {
  let level = get_string_value_or_default(value, "level", "unknown");

  let formatted_level = format!("{:>6.6}", level);

  let colour = match level.as_ref() {
    "info" => Colour::Green,
    "warn" => Colour::Yellow,
    "error" => Colour::Red,
    "debug" => Colour::Blue,
    _ => Colour::Purple
  };

  let message = get_string_value_or_default(value, "short_message", "");

  println!("{} | {}", colour.paint(formatted_level), message)
}
