use crate::log_settings::LogSettings;
use crate::time::try_convert_timestamp_to_readable;
use handlebars::Handlebars;
use serde_json::{Map, Value};
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::io::Write;
use yansi::Paint;

pub fn print_log_line(
	out: &mut dyn Write,
	maybe_prefix: Option<&str>,
	log_entry: &Map<String, Value>,
	log_settings: &LogSettings,
	handlebars: &Handlebars<'static>,
) {
	let string_log_entry = flatten_json(log_entry, "");
	let level = {
		let level = get_string_value_or_default(&string_log_entry, &log_settings.level_keys, "unknown");
		log_settings.level_map.get(&level).cloned().unwrap_or(level)
	};

	let trimmed_prefix = maybe_prefix.map(|p| p.trim()).unwrap_or_else(|| "").to_string();
	let mut message = get_string_value_or_default(&string_log_entry, &log_settings.message_keys, "");
	let timestamp = try_convert_timestamp_to_readable(get_string_value_or_default(&string_log_entry, &log_settings.time_keys, ""));

	if let Some(message_template) = &log_settings.substitution {
		if let Some(templated_message) = message_template.apply(&message, log_entry) {
			message = templated_message;
		}
	}

	let mut handle_bar_input: Map<String, Value> = log_entry.clone();
	handle_bar_input.insert("fblog_timestamp".to_string(), Value::String(timestamp));
	handle_bar_input.insert("fblog_level".to_string(), Value::String(level));
	handle_bar_input.insert("fblog_message".to_string(), Value::String(message));
	handle_bar_input.insert("fblog_prefix".to_string(), Value::String(trimmed_prefix));

	let write_result = match handlebars.render("main_line", &handle_bar_input) {
		Ok(string) => writeln!(out, "{}", string),
		Err(e) => writeln!(out, "{} Failed to process line: {}", "??? >".red().bold(), e),
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
					Err(e) => writeln!(out, "{} Failed to process additional value: {}", "   ??? >".red().bold(), e),
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
	use crate::template;

	fn without_style(styled: &str) -> String {
		use regex::Regex;
		let regex = Regex::new("\u{001B}\\[[\\d;]*[^\\d;]").expect("Regex should be valid");
		regex.replace_all(styled, "").into_owned()
	}

	fn fblog_handlebar_registry_default_format() -> Handlebars<'static> {
		let main_line_format = template::DEFAULT_MAIN_LINE_FORMAT.to_string();
		let additional_value_format = template::DEFAULT_ADDITIONAL_VALUE_FORMAT.to_string();

		template::fblog_handlebar_registry(main_line_format, additional_value_format)
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

		print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars);

		assert_eq!(out_to_string(out), "2017-07-06T15:21:16  INFO: something happend\n");
	}

	#[test]
	fn write_log_entry_with_mapped_level() {
		let handlebars = fblog_handlebar_registry_default_format();
		let mut log_settings = LogSettings::new_default_settings();
		log_settings.level_map = BTreeMap::from([("30".to_string(), "info".to_string())]);

		let mut out: Vec<u8> = Vec::new();
		let mut log_entry: Map<String, Value> = Map::new();
		log_entry.insert("message".to_string(), Value::String("something happend".to_string()));
		log_entry.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
		log_entry.insert("process".to_string(), Value::String("rust".to_string()));
		log_entry.insert("level".to_string(), Value::String("30".to_string()));

		print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars);

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

		print_log_line(&mut out, Some(prefix), &log_entry, &log_settings, &handlebars);

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

		print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars);

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

		print_log_line(&mut out, Some(prefix), &log_entry, &log_settings, &handlebars);

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
		print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars);

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

		print_log_line(&mut out, None, &log_entry, &log_settings, &handlebars);

		assert_eq!(out_to_string(out), "               moep  HUGO: rust\n");
	}
}
