use crate::filter;
use crate::log;
use crate::log_settings::LogSettings;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde_json::{Map, Value};
use std::io::Write;
use std::io::{self, BufRead};
use yansi::{Color, Paint};

lazy_static! {
	static ref ORANGE: Color = Color::Rgb(255, 135, 22);
}

pub fn process_input(
	log_settings: &LogSettings,
	input: &mut dyn io::BufRead,
	maybe_filter: Option<&String>,
	implicit_return: bool,
	handlebars: &Handlebars<'static>,
) {
	for line in input.lines() {
		match line {
			Ok(read_line) => match process_input_line(log_settings, &read_line, None, maybe_filter, implicit_return, handlebars) {
				Ok(_) => (),
				Err(_) => print_raw_line(&read_line, &ORANGE),
			},
			Err(e) => {
				print_raw_line(&format!("Could not read line: {e}"), &Color::Red);
			}
		}
	}
}

fn print_raw_line(line: &str, c: &Color) {
	let write_result = writeln!(&mut io::stdout(), "{} {}", "??? >".fg(*c).bold(), line);
	if write_result.is_err() {
		// Output end reached
		std::process::exit(14);
	}
}

fn process_input_line(
	log_settings: &LogSettings,
	read_line: &str,
	maybe_prefix: Option<&str>,
	maybe_filter: Option<&String>,
	implicit_return: bool,
	handlebars: &Handlebars<'static>,
) -> Result<(), ()> {
	match serde_json::from_str::<Value>(read_line) {
		Ok(Value::Object(log_entry)) => {
			process_json_log_entry(log_settings, maybe_prefix, &log_entry, maybe_filter, implicit_return, handlebars);
			Ok(())
		}
		_ => {
			if log_settings.with_prefix && maybe_prefix.is_none() {
				match read_line.find('{') {
					Some(pos) => {
						let prefix = &read_line[..pos];
						let rest = &read_line[pos..];
						process_input_line(log_settings, rest, Some(prefix), maybe_filter, implicit_return, handlebars)
					}
					None => Err(()),
				}
			} else {
				Err(())
			}
		}
	}
}

fn process_json_log_entry(
	log_settings: &LogSettings,
	maybe_prefix: Option<&str>,
	log_entry: &Map<String, Value>,
	maybe_filter: Option<&String>,
	implicit_return: bool,
	handlebars: &Handlebars<'static>,
) {
	if let Some(filter) = maybe_filter {
		match filter::show_log_entry(log_entry, filter, implicit_return, log_settings) {
			Ok(true) => process_log_entry(log_settings, maybe_prefix, log_entry, handlebars),
			Ok(false) => (),
			Err(e) => {
				writeln!(io::stderr(), "{}: '{:?}'", "Failed to apply filter expression".red(), e).expect("Should be able to write to stderr");
			}
		}
	} else {
		process_log_entry(log_settings, maybe_prefix, log_entry, handlebars)
	}
}

fn process_log_entry(log_settings: &LogSettings, maybe_prefix: Option<&str>, log_entry: &Map<String, Value>, handlebars: &Handlebars<'static>) {
	log::print_log_line(&mut io::stdout(), maybe_prefix, log_entry, log_settings, handlebars)
}
