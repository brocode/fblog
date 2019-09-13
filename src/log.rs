use ansi_term::{Colour, Style};
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::io::Write;

pub struct LogSettings {
  pub message_keys: Vec<String>,
  pub time_keys: Vec<String>,
  pub level_keys: Vec<String>,
  pub additional_values: Vec<String>,
  pub dump_all: bool,
  pub inspect: bool,
  pub with_prefix: bool,
}

impl LogSettings {
  pub fn new_default_settings() -> LogSettings {
    LogSettings {
      message_keys: vec!["short_message".to_string(), "msg".to_string(), "message".to_string()],
      time_keys: vec!["timestamp".to_string(), "time".to_string()],
      level_keys: vec!["level".to_string(), "severity".to_string()],
      additional_values: vec![],
      dump_all: false,
      inspect: false,
      with_prefix: false,
    }
  }

  pub fn add_additional_values(&mut self, mut additional_values: Vec<String>) {
    self.additional_values.append(&mut additional_values);
  }

  pub fn add_message_keys(&mut self, mut message_keys: Vec<String>) {
    message_keys.append(&mut self.message_keys);
    self.message_keys = message_keys;
  }

  pub fn add_time_keys(&mut self, mut time_keys: Vec<String>) {
    time_keys.append(&mut self.time_keys);
    self.time_keys = time_keys;
  }

  pub fn add_level_keys(&mut self, mut level_keys: Vec<String>) {
    level_keys.append(&mut self.level_keys);
    self.level_keys = level_keys;
  }
}

pub fn print_log_line(out: &mut dyn Write, maybe_prefix: Option<&str>, log_entry: &BTreeMap<String, String>, log_settings: &LogSettings) {
  let bold = Style::new().bold();

  let level = get_string_value_or_default(log_entry, &log_settings.level_keys, "unknown");

  let formatted_level = format!("{:>5.5}:", level.to_uppercase());

  let level_style = level_to_style(&level);
  let prefix_color = Colour::RGB(138, 43, 226).bold();
  let formated_prefix = maybe_prefix.map(|p| format!(" {}", prefix_color.paint(p))).unwrap_or_else(|| "".to_owned());
  let message = get_string_value_or_default(log_entry, &log_settings.message_keys, "");
  let timestamp = get_string_value_or_default(log_entry, &log_settings.time_keys, "");
  let painted_timestamp = bold.paint(format!("{:>19.19}", timestamp));

  writeln!(
    out,
    "{} {}{} {}",
    painted_timestamp,
    level_style.paint(formatted_level),
    formated_prefix,
    message
  )
  .expect("Expect to be able to write to out stream.");
  if log_settings.dump_all {
    let all_values: Vec<String> = log_entry.keys().map(ToOwned::to_owned).collect();
    write_additional_values(out, log_entry, &all_values);
  } else {
    write_additional_values(out, log_entry, &log_settings.additional_values);
  }
}

fn get_string_value(value: &BTreeMap<String, String>, keys: &[String]) -> Option<String> {
  keys
    .iter()
    .fold(None::<String>, |maybe_match, key| maybe_match.or_else(|| value.get(key).map(ToOwned::to_owned)))
}

fn get_string_value_or_default(value: &BTreeMap<String, String>, keys: &[String], default: &str) -> String {
  get_string_value(value, keys).unwrap_or_else(|| default.to_string())
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

fn write_additional_values(out: &mut dyn Write, log_entry: &BTreeMap<String, String>, additional_values: &[String]) {
  let bold_grey = Colour::RGB(150, 150, 150).bold();

  for additional_value in additional_values {
    if let Some(value) = get_string_value(log_entry, &[additional_value.to_string()]) {
      let trimmed_additional_value = format!("{:>25.25}:", additional_value.to_string());
      let painted_value = bold_grey.paint(trimmed_additional_value);
      writeln!(out, "{} {}", painted_value, value).expect("Expect to be able to write to out stream.");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use maplit::btreemap;
  use regex::Regex;

  fn out_to_string(out: Vec<u8>) -> String {
    let regex = Regex::new("\u{001B}\\[[\\d;]*[^\\d;]").expect("Regex should be valid");
    let out_with_style = String::from_utf8_lossy(&out).into_owned();
    let result = regex.replace_all(&out_with_style, "").into_owned();
    result
  }

  #[test]
  fn write_log_entry() {
    let log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "level".to_string() => "info".to_string()};

    print_log_line(&mut out, None, &log_entry, &log_settings);

    assert_eq!(out_to_string(out), "2017-07-06T15:21:16  INFO: something happend\n");
  }
  #[test]
  fn write_log_entry_with_prefix() {
    let log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let prefix = "abc";
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "level".to_string() => "info".to_string()};

    print_log_line(&mut out, Some(prefix), &log_entry, &log_settings);

    assert_eq!(out_to_string(out), "2017-07-06T15:21:16  INFO: abc something happend\n");
  }

  #[test]
  fn write_log_entry_with_additional_field() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()};
    let mut log_settings = LogSettings::new_default_settings();
    log_settings.add_additional_values(vec!["process".to_string(), "fu".to_string()]);

    print_log_line(&mut out, None, &log_entry, &log_settings);

    assert_eq!(
      out_to_string(out),
      "\
2017-07-06T15:21:16  INFO: something happend
                  process: rust
                       fu: bower
"
    );
  }
  #[test]
  fn write_log_entry_with_additional_field_and_prefix() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()};
    let prefix = "abc";
    let mut log_settings = LogSettings::new_default_settings();
    log_settings.add_additional_values(vec!["process".to_string(), "fu".to_string()]);

    print_log_line(&mut out, Some(prefix), &log_entry, &log_settings);

    assert_eq!(
      out_to_string(out),
      "\
2017-07-06T15:21:16  INFO: abc something happend
                  process: rust
                       fu: bower
"
    );
  }

  #[test]
  fn write_log_entry_dump_all() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()};

    let mut log_settings = LogSettings::new_default_settings();
    log_settings.dump_all = true;
    print_log_line(&mut out, None, &log_entry, &log_settings);

    assert_eq!(
      out_to_string(out),
      "\
2017-07-06T15:21:16  INFO: something happend
                       fu: bower
                    level: info
                  message: something happend
                  process: rust
                     time: 2017-07-06T15:21:16
"
    );
  }

  #[test]
  fn write_log_entry_with_exotic_fields() {
    let mut log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap! {"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "moep".to_string() => "moep".to_string(),
    "hugo".to_string() => "hugo".to_string(),
    "level".to_string() => "info".to_string()};

    log_settings.add_message_keys(vec!["process".to_string()]);
    log_settings.add_time_keys(vec!["moep".to_string()]);
    log_settings.add_level_keys(vec!["hugo".to_string()]);

    print_log_line(&mut out, None, &log_entry, &log_settings);

    assert_eq!(out_to_string(out), "               moep  HUGO: rust\n");
  }
}
