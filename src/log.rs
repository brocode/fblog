use ansi_term::{Colour, Style};
use std::collections::BTreeMap;
use std::io::Write;

pub fn print_log_line(out: &mut Write, log_entry: &BTreeMap<String, String>, additional_values: &[String], dump_all: bool) {
  let bold = Style::new().bold();

  let level = get_string_value_or_default(log_entry, &["level", "severity"], "unknown");

  let formatted_level = format!("{:>5.5}:", level.to_uppercase());

  let level_style = level_to_style(&level);

  let message = get_string_value_or_default(log_entry, &["short_message", "msg", "message"], "");
  let timestamp = get_string_value_or_default(log_entry, &["timestamp", "time"], "");
  let painted_timestamp = bold.paint(format!("{:>19.19}", timestamp));

  writeln!(out,
           "{} {} {}",
           painted_timestamp,
           level_style.paint(formatted_level),
           message)
    .expect("Expect to be able to write to out stream.");
  if dump_all {
    let all_values: Vec<String> = log_entry.keys().map(|k| k.to_owned()).collect();
    write_additional_values(out, log_entry, all_values.as_slice());
  } else {
    write_additional_values(out, log_entry, additional_values);
  }
}

fn get_string_value(value: &BTreeMap<String, String>, keys: &[&str]) -> Option<String> {
  keys.iter()
      .fold(None::<String>,
            |maybe_match, key| maybe_match.or_else(|| value.get(*key).map(|k| k.to_owned())))
}

fn get_string_value_or_default(value: &BTreeMap<String, String>, keys: &[&str], default: &str) -> String {
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



fn write_additional_values(out: &mut Write, log_entry: &BTreeMap<String, String>, additional_values: &[String]) {
  let bold_grey = Colour::RGB(150, 150, 150).bold();
  for additional_value in additional_values {
    if let Some(value) = get_string_value(log_entry, &[additional_value]) {
      let trimmed_additional_value = format!("{:>25.25}:", additional_value.to_string());
      let painted_value = bold_grey.paint(trimmed_additional_value);
      writeln!(out, "{} {}", painted_value, value).expect("Expect to be able to write to out stream.");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use regex::Regex;


  fn out_to_string(out: Vec<u8>) -> String {
    let regex = Regex::new("\u{001B}\\[[\\d;]*[^\\d;]").expect("Regex should be valid");
    let out_with_style = String::from_utf8_lossy(&out).into_owned();
    let result = regex.replace_all(&out_with_style, "").into_owned();
    // println!("Unclean result: '{}'", out_with_style);
    // println!("Clean result  : '{}'", result);
    result
  }

  #[test]
  fn write_log_entry() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap!{"message".to_string() => "something happend".to_string(),
                                                        "time".to_string() => "2017-07-06T15:21:16".to_string(),
                                                        "process".to_string() => "rust".to_string(),
                                                        "level".to_string() => "info".to_string()};

    print_log_line(&mut out, &log_entry, &[], false);

    assert_eq!(out_to_string(out),
               "2017-07-06T15:21:16  INFO: something happend\n");

  }

  #[test]
  fn write_log_entry_with_additional_field() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap!{"message".to_string() => "something happend".to_string(),
                                                        "time".to_string() => "2017-07-06T15:21:16".to_string(),
                                                        "process".to_string() => "rust".to_string(),
                                                        "fu".to_string() => "bower".to_string(),
                                                        "level".to_string() => "info".to_string()};

    print_log_line(&mut out,
                   &log_entry,
                   &["process".to_string(), "fu".to_string()],
                   false);

    assert_eq!(out_to_string(out),
               "\
2017-07-06T15:21:16  INFO: something happend
                  process: rust
                       fu: bower
");
  }

  #[test]
  fn write_log_entry_dump_all() {
    let mut out: Vec<u8> = Vec::new();
    let log_entry: BTreeMap<String, String> = btreemap!{"message".to_string() => "something happend".to_string(),
                                                        "time".to_string() => "2017-07-06T15:21:16".to_string(),
                                                        "process".to_string() => "rust".to_string(),
                                                        "fu".to_string() => "bower".to_string(),
                                                        "level".to_string() => "info".to_string()};

    print_log_line(&mut out, &log_entry, &[], true);

    assert_eq!(out_to_string(out), "\
2017-07-06T15:21:16  INFO: something happend
                       fu: bower
                    level: info
                  message: something happend
                  process: rust
                     time: 2017-07-06T15:21:16
");


  }
}
