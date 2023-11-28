use regex::{Captures, Regex};
use serde_json::Value;
use yansi::Color;

use crate::no_color_support::stylew;

#[derive(Debug)]
pub enum Error {
    MissingIdentifier,
    RegexParse(regex::Error),
}

impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self {
        Self::RegexParse(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingIdentifier => f.write_str("The identifier `key` is missing"),
            Self::RegexParse(rx) => f.write_fmt(format_args!("Regular expression could not be created for format: {}", rx)),
        }
    }
}

/// 7 bytes for color (`\e` `[` `1` `;` `3` `9` `m`) and 4 bytes for reset (`\e` `[` `0` `m`)
const COLOR_OVERHEAD: usize = 7 + 4;

pub struct Substitution {
    pub context_key: String,
    placeholder_prefix: String,
    placeholder_suffix: String,
    placeholder_regex: Regex,
}

impl Substitution {
    pub const DEFAULT_PLACEHOLDER_FORMAT: &'static str = "{key}";
    pub const KEY_DELIMITER: &'static str = "key";
    pub const DEFAULT_CONTEXT_KEY: &'static str = "context";

    pub fn new<S: Into<String>>(context_key: Option<S>, placeholder_format: Option<S>) -> Result<Self, Error> {
        let format = placeholder_format.map_or(Self::DEFAULT_PLACEHOLDER_FORMAT.to_owned(), Into::into);
        let (prefix, suffix) = Self::parse_placeholder_format(&format)?;

        let placeholder_regex = Self::create_regex(prefix, suffix)?;

        Ok(Self {
            context_key: context_key.map_or(Self::DEFAULT_CONTEXT_KEY.to_owned(), Into::into),
            placeholder_prefix: prefix.to_owned(),
            placeholder_suffix: suffix.to_owned(),
            placeholder_regex,
        })
    }

    fn parse_placeholder_format(format: &str) -> Result<(&str, &str), Error> {
        format.split_once(Self::KEY_DELIMITER).ok_or(Error::MissingIdentifier)
    }

    fn create_regex(prefix: &str, suffix: &str) -> Result<Regex, regex::Error> {
        Regex::new(&format!("{}([a-z|A-Z|0-9|_|-]+){}", regex::escape(prefix), regex::escape(suffix)))
    }

    pub(crate) fn apply(&self, message: &str, log_entry: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
        let Some(context_value) = log_entry.get(&self.context_key) else {
            return None;
        };

        let key_format_overhead = self.placeholder_prefix.len() + COLOR_OVERHEAD + self.placeholder_suffix.len() + COLOR_OVERHEAD;

        return Some(
            self.placeholder_regex
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
                            stylew(&mut buf, &Color::Default.style().dimmed(), &self.placeholder_prefix);
                            stylew(&mut buf, &Color::Red.style().bold(), key);
                            stylew(&mut buf, &Color::Default.style().dimmed(), &self.placeholder_suffix);
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
                stylew(&mut buf, &Color::Default.style().dimmed(), ", ");
            }
            self.color_format(buf, value);
        }
        stylew(buf, &Color::Default.style().dimmed(), "]");
    }

    fn color_format_map(&self, mut buf: &mut String, o: &serde_json::Map<String, Value>) {
        stylew(&mut buf, &Color::Default.style().dimmed(), "{");
        for (i, (key, value)) in o.iter().enumerate() {
            if i > 0 {
                stylew(&mut buf, &Color::Default.style().dimmed(), ", ");
            }
            stylew(&mut buf, &Color::Magenta.style(), key);
            stylew(&mut buf, &Color::Default.style().dimmed(), ": ");
            self.color_format(buf, value);
        }
        stylew(buf, &Color::Default.style().dimmed(), "}");
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::new::<String>(None, None).expect("default placeholder should parse")
    }
}

#[cfg(test)]
mod tests {
    use crate::no_color_support::without_style;

    use super::*;
    type JMap = serde_json::Map<String, serde_json::Value>;

    fn entry_context<V: Into<serde_json::Value>>(subst: &Substitution, context: V) -> JMap {
        let mut map = serde_json::Map::new();
        map.insert(subst.context_key.clone(), context.into());
        map
    }

    #[test]
    fn common_placeholder_formats() {
        test_placeholder_format("{key}");
        test_placeholder_format("[key]");
        test_placeholder_format("%key%");
        test_placeholder_format("${key}");
    }

    fn test_placeholder_format(placeholder: &str) {
        let subst = Substitution::new(None, Some(placeholder)).unwrap();
        let msg = format!("Tapping fingers as a way to {placeholder}");
        let mut context = serde_json::Map::new();
        context.insert("key".into(), "speak".into());

        let result = subst.apply(&msg, &entry_context(&subst, context)).unwrap_or(msg);
        assert_eq!(
            "Tapping fingers as a way to speak",
            without_style(&result),
            "Failed to substitute with placeholder format {}",
            placeholder
        );
    }

    #[test]
    fn placeholder_not_in_context() {
        let subst = Substitution::default();
        let msg = "substituted: {subst}, ignored: {ignored}";
        let mut context = serde_json::Map::new();
        context.insert("subst".into(), "no brackets!".into());

        let result = subst.apply(msg, &entry_context(&subst, context)).unwrap_or(msg.to_owned());
        assert_eq!("substituted: no brackets!, ignored: {ignored}", without_style(&result));
    }

    #[test]
    fn array_context() {
        let subst = Substitution::default();
        let msg = "text: {0}, number: {1}, bool: {2}, ignored: {3}";
        let context: Vec<Value> = vec!["better than sleeping".into(), 9.into(), true.into()];

        let result = subst.apply(msg, &entry_context(&subst, context)).unwrap_or(msg.to_owned());
        assert_eq!("text: better than sleeping, number: 9, bool: true, ignored: {3}", without_style(&result));
    }
}
