use crate::no_color_support::style;
use crate::substitution::Substitution;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::io::Write;
use yansi::Color;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct LogSettings {
  pub message_keys: Vec<String>,
  pub time_keys: Vec<String>,
  pub level_keys: Vec<String>,
  pub additional_values: Vec<String>,
  pub excluded_values: Vec<String>,
  pub dump_all: bool,
  pub with_prefix: bool,
  pub print_lua: bool,
  pub substitution_enabled: bool,
  pub context_keys: Vec<String>,
  pub placeholder_format: String,
}

impl LogSettings {
  const DEFAULT_MESSAGE_KEYS: [&str; 3] = ["short_message", "msg", "message"];
  const DEFAULT_TIME_KEYS: [&str; 3] = ["timestamp", "time", "@timestamp"];
  const DEFAULT_LEVEL_KEYS: [&str; 4] = ["level", "severity", "log.level", "loglevel"];

  #[cfg(test)]
  pub fn new_default_settings() -> LogSettings {
    let mut log_settings = LogSettings {
      message_keys: vec![],
      time_keys: vec![],
      level_keys: vec![],
      additional_values: vec![],
      excluded_values: vec![],
      dump_all: false,
      with_prefix: false,
      print_lua: false,
      substitution_enabled: false,
      context_keys: vec![],
      placeholder_format: Substitution::DEFAULT_PLACEHOLDER_FORMAT.to_owned(),
    };
    log_settings.add_default_keys();
    log_settings
  }

  pub fn add_default_keys(&mut self) {
    self.message_keys.extend(Self::DEFAULT_MESSAGE_KEYS.into_iter().map(ToOwned::to_owned));
    self.time_keys.extend(Self::DEFAULT_TIME_KEYS.into_iter().map(ToOwned::to_owned));
    self.level_keys.extend(Self::DEFAULT_LEVEL_KEYS.into_iter().map(ToOwned::to_owned));
    self.context_keys.extend(Substitution::DEFAULT_CONTEXT_KEYS.into_iter().map(ToOwned::to_owned));
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

  pub fn add_excluded_values(&mut self, mut excluded_values: Vec<String>) {
    self.excluded_values.append(&mut excluded_values);
  }

  pub fn add_context_keys(&mut self, mut context_keys: Vec<String>) {
    context_keys.append(&mut self.context_keys);
    self.context_keys = context_keys;
  }
}

pub fn print_log_line(
  out: &mut dyn Write,
  maybe_prefix: Option<&str>,
  log_entry: &Map<String, Value>,
  log_settings: &LogSettings,
  handlebars: &Handlebars<'static>,
  substitution: Option<&Substitution>,
) {
  let string_log_entry = flatten_json(log_entry, "");
  let level = get_string_value_or_default(&string_log_entry, &log_settings.level_keys, "unknown");

  let formatted_prefix = maybe_prefix.map(|p| format!(" {}", p)).unwrap_or_else(|| "".to_owned());
  let mut message = get_string_value_or_default(&string_log_entry, &log_settings.message_keys, "");
  let timestamp = get_string_value_or_default(&string_log_entry, &log_settings.time_keys, "");

  if let Some(substitution) = substitution {
    if let Some(templated_message) = substitution.apply(&message, log_entry) {
      message = templated_message;
    }
  }

  let mut handle_bar_input: Map<String, Value> = log_entry.clone();
  handle_bar_input.insert("fblog_timestamp".to_string(), Value::String(timestamp));
  handle_bar_input.insert("fblog_level".to_string(), Value::String(level));
  handle_bar_input.insert("fblog_message".to_string(), Value::String(message));
  handle_bar_input.insert("fblog_prefix".to_string(), Value::String(formatted_prefix));

  let write_result = match handlebars.render("main_line", &handle_bar_input) {
    Ok(string) => writeln!(out, "{}", string),
    Err(e) => writeln!(out, "{} Failed to process line: {}", style(&Color::Red.style().bold(), "??? >"), e),
  };

  if write_result.is_err() {
    // Output end reached
    std::process::exit(14);
  }

  if log_settings.dump_all {
    let all_values: Vec<String> = string_log_entry
      .keys()
      .map(ToOwned::to_owned)
      .filter(|v| !log_settings.excluded_values.contains(v))
      .collect();
    write_additional_values(out, &string_log_entry, &all_values, handlebars);
  } else {
    write_additional_values(out, &string_log_entry, &log_settings.additional_values, handlebars);
  }
}

fn flatten_json(log_entry: &Map<String, Value>, prefix: &str) -> BTreeMap<String, String> {
  let mut flattened_json: BTreeMap<String, String> = BTreeMap::new();
  for (key, value) in log_entry {
    match value {
      Value::String(ref string_value) => {
        flattened_json.insert(format!("{}{}", prefix, key), string_value.to_string());
      }
      Value::Bool(ref bool_value) => {
        flattened_json.insert(format!("{}{}", prefix, key), bool_value.to_string());
      }
      Value::Number(ref number_value) => {
        flattened_json.insert(format!("{}{}", prefix, key), number_value.to_string());
      }
      Value::Array(ref array_values) => {
        for (index, array_value) in array_values.iter().enumerate() {
          let key = format!("{}[{}]", key, index + 1); // lua tables indexes start with 1

          match array_value {
            Value::Array(array_values) => {
              flatten_array(&key, prefix, array_values, &mut flattened_json);
            }
            Value::Object(nested_entry) => {
              flattened_json.extend(flatten_json(nested_entry, &format!("{}{} > ", prefix, key)));
            }
            _ => {
              flattened_json.insert(format!("{}{}", prefix, key), array_value.to_string());
            }
          };
        }
      }
      Value::Object(nested_entry) => {
        flattened_json.extend(flatten_json(nested_entry, &format!("{}{} > ", prefix, key)));
      }
      Value::Null => {}
    };
  }
  flattened_json
}

fn flatten_array(key: &str, prefix: &str, array_values: &[Value], flattened_json: &mut BTreeMap<String, String>) {
  for (index, array_value) in array_values.iter().enumerate() {
    let key = format!("{}[{}]", key, index + 1); // lua tables indexes start with 1

    match array_value {
      Value::Array(nested_array_values) => flatten_array(&key, prefix, nested_array_values, flattened_json),
      Value::Object(nested_entry) => {
        flattened_json.extend(flatten_json(nested_entry, &format!("{}{} > ", prefix, key)));
      }
      _ => {
        flattened_json.insert(format!("{}{}", prefix, key), array_value.to_string());
      }
    };
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

fn write_additional_values(out: &mut dyn Write, log_entry: &BTreeMap<String, String>, additional_values: &[String], handlebars: &Handlebars<'static>) {
  for additional_value_prefix in additional_values {
    for additional_value in log_entry
      .keys()
      .filter(|k| *k == additional_value_prefix || k.starts_with(&format!("{}{}", additional_value_prefix, " > ")))
    {
      if let Some(value) = get_string_value(log_entry, &[additional_value.to_string()]) {
        let mut variables: BTreeMap<String, String> = BTreeMap::new();
        variables.insert("key".to_string(), additional_value.to_string());
        variables.insert("value".to_string(), value.to_string());

        let write_result = match handlebars.render("additional_value", &variables) {
          Ok(string) => writeln!(out, "{}", string),
          Err(e) => writeln!(
            out,
            "{} Failed to process additional value: {}",
            style(&Color::Red.style().bold(), "   ??? >"),
            e
          ),
        };
        if write_result.is_err() {
          // Output end reached
          std::process::exit(14);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{no_color_support::without_style, template};

  fn fblog_handlebar_registry_default_format() -> Handlebars<'static> {
    let main_line_format = template::DEFAULT_MAIN_LINE_FORMAT.to_string();
    let additional_value_format = template::DEFAULT_ADDITIONAL_VALUE_FORMAT.to_string();

    template::fblog_handlebar_registry(&template::Settings {
      main_line_format,
      additional_value_format,
    })
  }

  fn out_to_string(out: Vec<u8>) -> String {
    let out_with_style = String::from_utf8_lossy(&out).into_owned();
    without_style(&out_with_style)
  }

  #[test]
  fn write_log_entry() {
    let handlebars = fblog_handlebar_registry_default_format();
    let log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));

    print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars, None);

    assert_eq!(out_to_string(out), "2017-07-06T15:21:16  INFO: something happend\n");
  }

  #[test]
  fn write_log_entry_with_prefix() {
    let handlebars = fblog_handlebar_registry_default_format();
    let log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let prefix = "abc";
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));

    print_log_line(&mut out, Some(prefix), &log_entry, &log_settings, &handlebars, None);

    assert_eq!(out_to_string(out), "2017-07-06T15:21:16  INFO: abc something happend\n");
  }

  #[test]
  fn write_log_entry_with_additional_field() {
    let handlebars = fblog_handlebar_registry_default_format();
    let mut out: Vec<u8> = Vec::new();
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("fu".to_string(), Value::String("bower".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));
    let mut log_settings = LogSettings::new_default_settings();
    log_settings.add_additional_values(vec!["process".to_string(), "fu".to_string()]);

    print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars, None);

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
    let handlebars = fblog_handlebar_registry_default_format();
    let mut out: Vec<u8> = Vec::new();
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("fu".to_string(), Value::String("bower".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));

    let prefix = "abc";
    let mut log_settings = LogSettings::new_default_settings();
    log_settings.add_additional_values(vec!["process".to_string(), "fu".to_string()]);

    print_log_line(&mut out, Some(prefix), &log_entry, &log_settings, &handlebars, None);

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
    let handlebars = fblog_handlebar_registry_default_format();
    let mut out: Vec<u8> = Vec::new();
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("fu".to_string(), Value::String("bower".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));

    let mut log_settings = LogSettings::new_default_settings();
    log_settings.dump_all = true;
    print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars, None);

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
    let handlebars = fblog_handlebar_registry_default_format();
    let mut log_settings = LogSettings::new_default_settings();
    let mut out: Vec<u8> = Vec::new();
    let mut log_entry: Map<String, Value> = Map::new();
    log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
    log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    log_entry.insert("process".to_string(), Value::String("rust".to_string()));
    log_entry.insert("moep".to_string(), Value::String("moep".to_string()));
    log_entry.insert("hugo".to_string(), Value::String("hugo".to_string()));
    log_entry.insert("level".to_string(), Value::String("info".to_string()));

    log_settings.add_message_keys(vec!["process".to_string()]);
    log_settings.add_time_keys(vec!["moep".to_string()]);
    log_settings.add_level_keys(vec!["hugo".to_string()]);

    print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars, None);

    assert_eq!(out_to_string(out), "               moep  HUGO: rust\n");
  }
}
