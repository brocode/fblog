use std::fmt;

use regex::{Captures, Regex};
use serde_json::Value;
use yansi::Paint;

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
		let context_value = log_entry.get(&self.context_key)?;

		Some(
			self
				.placeholder_regex
				.replace_all(message, |caps: &Captures| {
					let key = &caps[1];
					let value = match context_value {
						Value::Object(o) => o.get(key),
						Value::Array(a) => key.parse().map(|i| a.get::<usize>(i)).unwrap_or(None),
						_ => None,
					};
					match value {
						None => {
							format!("{}{}{}", self.placeholder_prefix.dim(), key.red().bold(), self.placeholder_suffix.dim())
						}
						Some(value) => {
							let mut buf = String::new();
							let _ = self.color_format(&mut buf, value);
							buf
						}
					}
				})
				.into_owned(),
		)
	}

	fn color_format<W: std::fmt::Write>(&self, buf: &mut W, value: &Value) -> Result<(), fmt::Error> {
		match value {
			Value::String(s) => write!(buf, "{}", s.yellow().bold()),
			Value::Number(n) => write!(buf, "{}", n.to_string().cyan().bold()),
			Value::Array(a) => self.color_format_array(buf, a),
			Value::Object(o) => self.color_format_map(buf, o),
			Value::Bool(true) => write!(buf, "{}", "true".green().bold()),
			Value::Bool(false) => write!(buf, "{}", "false".red().bold()),
			Value::Null => write!(buf, "{}", "null".bold()),
		}?;
		Ok(())
	}

	fn color_format_array<W: std::fmt::Write>(&self, buf: &mut W, a: &[Value]) -> Result<(), fmt::Error> {
		write!(buf, "{}", "[".dim())?;
		for (i, value) in a.iter().enumerate() {
			if i > 0 {
				write!(buf, "{}", ", ".dim())?;
			}
			self.color_format(buf, value)?;
		}
		write!(buf, "{}", "]".dim())?;
		Ok(())
	}

	fn color_format_map<W: std::fmt::Write>(&self, buf: &mut W, o: &serde_json::Map<String, Value>) -> Result<(), fmt::Error> {
		write!(buf, "{}", "{".dim())?;
		for (i, (key, value)) in o.iter().enumerate() {
			if i > 0 {
				write!(buf, "{}", ", ".dim())?;
			}
			write!(buf, "{}", key.magenta())?;
			write!(buf, "{}", ": ".dim())?;
			self.color_format(buf, value)?;
		}
		write!(buf, "{}", "}".dim())?;
		Ok(())
	}
}

impl Default for Substitution {
	fn default() -> Self {
		Self::new::<String>(None, None).expect("default placeholder should parse")
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	type JMap = serde_json::Map<String, serde_json::Value>;

	fn without_style(styled: &str) -> String {
		let regex = Regex::new("\u{001B}\\[[\\d;]*[^\\d;]").expect("Regex should be valid");
		regex.replace_all(styled, "").into_owned()
	}

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
