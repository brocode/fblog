use std::fmt::Write;

use regex::{Captures, Regex};
use serde_json::Value;
use yansi::Color;

use crate::no_color_support::stylew;

#[derive(Debug)]
pub enum FormatError {
  MissingIdentifier,
  RegexParse(regex::Error),
}

impl From<regex::Error> for FormatError {
  fn from(value: regex::Error) -> Self {
    Self::RegexParse(value)
  }
}

impl std::fmt::Display for FormatError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingIdentifier => f.write_str("The identifier `key` is missing"),
      Self::RegexParse(rx) => f.write_fmt(format_args!("Regular expression could not be created for format: {}", rx)),
    }
  }
}

/// 7 bytes for color (`\e` `[` `1` `;` `3` `9` `m`) and 4 bytes for reset (`\e` `[` `0` `m`)
const COLOR_OVERHEAD: usize = 7 + 4;

pub struct MessageTemplate {
  pub context: String,
  key_prefix: String,
  key_suffix: String,
  key_regex: Regex,
}

impl MessageTemplate {
  const DEFAULT_KEY_PREFIX: &str = "{";
  const DEFAULT_KEY_SUFFIX: &str = "}";
  const DEFAULT_CONTEXT: &str = "context";

  pub fn new(context: String) -> Self {
    let key_prefix = Self::DEFAULT_KEY_PREFIX.to_owned();
    let key_suffix = Self::DEFAULT_KEY_SUFFIX.to_owned();
    let key_regex = Self::create_regex(&key_prefix, &key_suffix).expect("default key should compile");
    Self {
      context,
      key_prefix,
      key_suffix,
      key_regex,
    }
  }

  pub fn set_key_format(&mut self, format: &str) -> Result<(), FormatError> {
    let (prefix, suffix) = format.split_once("key").ok_or(FormatError::MissingIdentifier)?;
    self.key_regex = Self::create_regex(prefix, suffix)?;
    self.key_prefix = prefix.to_owned();
    self.key_suffix = suffix.to_owned();
    Ok(())
  }

  pub fn with_key_format(mut self, format: &str) -> Result<Self, FormatError> {
    self.set_key_format(format)?;
    Ok(self)
  }

  fn create_regex(prefix: &str, suffix: &str) -> Result<Regex, regex::Error> {
    Regex::new(&format!("{}([a-z|A-Z|0-9|_|-]+){}", regex::escape(prefix), regex::escape(suffix)))
  }

  pub(crate) fn apply(&self, message: &str, log_entry: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    let Some(context_value) = log_entry.get(&self.context) else {
      return None;
    };

    let key_format_overhead = self.key_prefix.len() + COLOR_OVERHEAD + self.key_suffix.len() + COLOR_OVERHEAD;

    return Some(
      self
        .key_regex
        .replace_all(message, |caps: &Captures| {
          let key = &caps[1];
          let value = match context_value {
            Value::Object(o) => o.get(key),
            Value::Array(a) => key.parse().map(|i| a.get::<usize>(i)).unwrap_or(None),
            _ => None,
          };
          match value {
            None => {
              let mut buf = String::with_capacity(key.len() + COLOR_OVERHEAD + key_format_overhead);
              stylew(&mut buf, &Color::Default.style().dimmed(), &self.key_prefix);
              stylew(&mut buf, &Color::Red.style().bold(), key);
              stylew(&mut buf, &Color::Default.style().dimmed(), &self.key_suffix);
              buf
            }
            Some(value) => {
              let mut buf = String::new();
              self.color_format(&mut buf, value);
              buf
            }
          }
        })
        .into_owned(),
    );
  }

  fn color_format(&self, buf: &mut String, value: &Value) {
    match value {
      Value::String(s) => stylew(buf, &Color::Yellow.style().bold(), s),
      Value::Number(n) => stylew(buf, &Color::Cyan.style().bold(), &n.to_string()),
      Value::Array(a) => self.color_format_array(buf, a),
      Value::Object(o) => self.color_format_map(buf, o),
      Value::Bool(true) => stylew(buf, &Color::Green.style().bold(), "true"),
      Value::Bool(false) => stylew(buf, &Color::Red.style().bold(), "false"),
      Value::Null => stylew(buf, &Color::Default.style().bold(), "null"),
    }
  }

  fn color_format_array(&self, mut buf: &mut String, a: &[Value]) {
    stylew(&mut buf, &Color::Default.style().dimmed(), "[");
    for (i, value) in a.iter().enumerate() {
      if i > 0 {
        _ = buf.write_str(", ");
      }
      self.color_format(buf, value);
    }
    stylew(buf, &Color::Default.style().dimmed(), "]");
  }

  fn color_format_map(&self, mut buf: &mut String, o: &serde_json::Map<String, Value>) {
    stylew(&mut buf, &Color::Default.style().dimmed(), "{");
    for (i, (key, value)) in o.iter().enumerate() {
      if i > 0 {
        _ = buf.write_char(',');
      }
      _ = buf.write_char(' ');
      stylew(&mut buf, &Color::Magenta.style(), key);
      stylew(&mut buf, &Color::Default.style().dimmed(), ": ");
      self.color_format(buf, value);
    }
    stylew(buf, &Color::Default.style().dimmed(), " }");
  }
}

impl Default for MessageTemplate {
  fn default() -> Self {
    Self::new(Self::DEFAULT_CONTEXT.into())
  }
}
