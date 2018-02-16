use hlua::{Lua, LuaError};
use std::collections::BTreeMap;

pub fn show_log_entry(log_entry: &BTreeMap<String, String>, filter_expr: &str, implicit_return: bool) -> Result<bool, LuaError> {
  let mut lua = Lua::new();
  lua.openlibs();

  for (key, value) in log_entry {
    lua.set(key.to_owned(), value.to_owned());
  }

  if implicit_return {
    lua.execute(&format!("return {};", filter_expr))
  } else {
    lua.execute(filter_expr)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_log_entry() -> BTreeMap<String, String> {
    btreemap!{"message".to_string() => "something happend".to_string(),
    "time".to_string() => "2017-07-06T15:21:16".to_string(),
    "process".to_string() => "rust".to_string(),
    "fu".to_string() => "bower".to_string(),
    "level".to_string() => "info".to_string()}
  }

  #[test]
  fn allow_all() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, "true", true).unwrap());
  }

  #[test]
  fn deny_all() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(false, show_log_entry(&log_entry, "false", true).unwrap());
  }

  #[test]
  fn filter_process() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust""#, true).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"process == "meep""#, true).unwrap()
    );
  }

  #[test]
  fn filter_logical_operators() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust" and fu == "bower""#, true).unwrap()
    );
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust" or fu == "bauer""#, true).unwrap()
    );
  }

  #[test]
  fn filter_contains() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(
        &log_entry,
        r#"string.find(message, "something") ~= nil"#,
        true
      ).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"string.find(message, "bla") ~= nil"#, true).unwrap()
    );
  }

  #[test]
  fn filter_regex() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"string.find(fu, "bow.*") ~= nil"#, true).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"string.find(fu, "bow.*sd") ~= nil"#, true).unwrap()
    );
  }

  #[test]
  fn unknown_variable() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      false,
      show_log_entry(
        &log_entry,
        r#"sdkfjsdfjsf ~= nil and string.find(sdkfjsdfjsf, "bow.*") ~= nil"#,
        true
      ).unwrap()
    );
  }

  #[test]
  fn no_implicit_return() {
    let log_entry: BTreeMap<String, String> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(
        &log_entry,
        r#"if 3 > 2 then return true else return false end"#,
        false
      ).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(
        &log_entry,
        r#"if 1 > 2 then return true else return false end"#,
        false
      ).unwrap()
    );
  }

}
