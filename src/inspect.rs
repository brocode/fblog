use std::collections::BTreeMap;
use std::collections::HashSet;
use std::io::Write;

pub struct InspectLogger {
  keys: HashSet<String>,
}

impl InspectLogger {
  pub fn new() -> InspectLogger {
    InspectLogger { keys: HashSet::new() }
  }

  pub fn print_unknown_keys(&mut self, log_entry: &BTreeMap<String, String>, write: &mut Write) {
    for entry in log_entry.keys() {
      if !self.keys.contains(entry) {
        writeln!(write, "{}", entry).expect("default should be stdout");
        self.keys.insert(entry.to_string());
      }
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
    result
  }

  #[test]
  fn inspect_log_entry() {
    let mut inpect_logger = InspectLogger::new();
    let mut out: Vec<u8> = Vec::new();

    let mut log_entry: BTreeMap<String, String> = btreemap!{"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()};
    inpect_logger.print_unknown_keys(&log_entry, &mut out);

    let result = out_to_string(out);
    assert_eq!(result, "fu\nlevel\nmessage\nprocess\ntime\n");

    let mut out: Vec<u8> = Vec::new();
    inpect_logger.print_unknown_keys(&log_entry, &mut out);
    let result = out_to_string(out);
    assert_eq!(result, "");

    log_entry.insert("sxoe".to_string(), "kuci".to_string());
    let mut out: Vec<u8> = Vec::new();
    inpect_logger.print_unknown_keys(&log_entry, &mut out);
    let result = out_to_string(out);
    assert_eq!(result, "sxoe\n");

    log_entry.insert("fkbr".to_string(), "kuci".to_string());
    log_entry.insert("blubb".to_string(), "kuci".to_string());
    let mut out: Vec<u8> = Vec::new();
    inpect_logger.print_unknown_keys(&log_entry, &mut out);
    let result = out_to_string(out);
    assert_eq!(result, "blubb\nfkbr\n");
  }
}
