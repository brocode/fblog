use crate::log::LogSettings;
use lazy_static::lazy_static;
use mlua::{Error as LuaError, Lua};
use regex::Regex;
use serde_json::{Map, Value};
use std::fmt::Write as _;

lazy_static! {
  static ref LUA_IDENTIFIER_CLEANUP: Regex = Regex::new(r"[^A-Za-z_]").unwrap();
  static ref LUA_STRING_ESCAPE: Regex = Regex::new(r"([\n])").unwrap();
}

pub fn show_log_entry(log_entry: &Map<String, Value>, filter_expr: &str, implicit_return: bool, log_settings: &LogSettings) -> Result<bool, LuaError> {
  let lua = Lua::new();

  let script = object_to_record(log_entry, false);
  if log_settings.print_lua {
    println!("{}", script);
  }
  lua.load(&script).exec()?;

  if implicit_return {
    lua.load(&format!("return {};", filter_expr)).eval::<bool>()
  } else {
    lua.load(filter_expr).eval::<bool>()
  }
}

fn object_to_record(object: &Map<String, Value>, nested: bool) -> String {
  let lines: Vec<String> = object
    .iter()
    .map(|(key, value)| {
      let mut script = String::new();
      let key_name = LUA_IDENTIFIER_CLEANUP.replace_all(key, "_");
      write!(script, "{} = ", key_name).expect("Should be able to write to string");
      match value {
        Value::String(ref string_value) => write!(script, "\"{}\"", escape_lua_string(string_value)).expect("Should be able to write to string"),
        Value::Bool(ref bool_value) => script.push_str(&bool_value.to_string()),
        Value::Number(ref number_value) => script.push_str(&number_value.to_string()),
        Value::Object(nested_object) => {
          let object_string = object_to_record(nested_object, true);
          write!(script, "{{{}}}", object_string).expect("Should be able to write to string");
        }
        Value::Array(array_values) => {
          let mut values = vec![];

          for array_value in array_values.iter() {
            let lua_array_value = match array_value {
              Value::String(ref string_value) => format!("\"{}\"", escape_lua_string(string_value)),
              Value::Bool(ref bool_value) => bool_value.to_string(),
              Value::Number(ref number_value) => number_value.to_string(),
              _ => "\"unsupported\"".to_string(),
            };

            values.push(lua_array_value);
          }

          write!(script, "{{{}}}", values.join(",")).expect("Should be able to write to string");
        }
        _ => write!(script, "\"unsupported\"").expect("Should be able to write to string"),
      }
      script
    })
    .collect();
  lines.join(if nested { "," } else { "\n" })
}

fn escape_lua_string(src: &str) -> String {
  let mut escaped = String::with_capacity(src.len());
  for c in src.chars() {
    match c {
      '\n' => escaped += "\\n",
      '\r' => escaped += "\\r",
      '\t' => escaped += "\\t",
      '"' => escaped += "\\\"",
      '\'' => escaped += "\\'",
      '\\' => escaped += "\\\\",
      c => write!(escaped, "{}", c).expect("Should be able to write to string"),
    }
  }
  escaped
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_log_entry() -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("message".to_string(), Value::String("something happend".to_string()));
    map.insert("time".to_string(), Value::String("2017-07-06T15:21:16".to_string()));
    map.insert("process".to_string(), Value::String("rust".to_string()));
    map.insert("fu".to_string(), Value::String("bower".to_string()));
    map.insert("level".to_string(), Value::String("info".to_string()));

    let mut nested = Map::new();
    nested.insert("log.level".to_string(), Value::String("debug".to_string()));

    map.insert("nested".to_string(), Value::Object(nested));

    let mut nested_with_array = Map::new();
    nested_with_array.insert(
      "array".to_string(),
      Value::Array(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
      ]),
    );

    map.insert("nested_with_array".to_string(), Value::Object(nested_with_array));

    map
  }

  #[test]
  fn allow_all() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(true, show_log_entry(&log_entry, "true", true, &LogSettings::new_default_settings()).unwrap());
  }

  #[test]
  fn deny_all() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(false, show_log_entry(&log_entry, "false", true, &LogSettings::new_default_settings()).unwrap());
  }

  #[test]
  fn filter_process() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust""#, true, &LogSettings::new_default_settings()).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"process == "meep""#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }

  #[test]
  fn filter_logical_operators() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust" and fu == "bower""#, true, &LogSettings::new_default_settings()).unwrap()
    );
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"process == "rust" or fu == "bauer""#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }

  #[test]
  fn filter_contains() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(
        &log_entry,
        r#"string.find(message, "something") ~= nil"#,
        true,
        &LogSettings::new_default_settings()
      )
      .unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"string.find(message, "bla") ~= nil"#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }

  #[test]
  fn filter_regex() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(
      true,
      show_log_entry(&log_entry, r#"string.find(fu, "bow.*") ~= nil"#, true, &LogSettings::new_default_settings()).unwrap()
    );
    assert_eq!(
      false,
      show_log_entry(&log_entry, r#"string.find(fu, "bow.*sd") ~= nil"#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }

  #[test]
  fn unknown_variable() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert_eq!(
      false,
      show_log_entry(
        &log_entry,
        r#"sdkfjsdfjsf ~= nil and string.find(sdkfjsdfjsf, "bow.*") ~= nil"#,
        true,
        &LogSettings::new_default_settings()
      )
      .unwrap()
    );
  }

  #[test]
  fn no_implicit_return() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert!(
      show_log_entry(
        &log_entry,
        r#"if 3 > 2 then return true else return false end"#,
        false,
        &LogSettings::new_default_settings()
      )
      .unwrap()
    );
    assert!(
      !show_log_entry(
        &log_entry,
        r#"if 1 > 2 then return true else return false end"#,
        false,
        &LogSettings::new_default_settings()
      )
      .unwrap()
    );
  }

  #[test]
  fn neted() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert!(
      show_log_entry(&log_entry, r#"nested.log_level == "debug""#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }

  #[test]
  fn nested_with_array() {
    let log_entry: Map<String, Value> = test_log_entry();
    assert!(
      show_log_entry(&log_entry, r#"nested_with_array.array[2] == "b""#, true, &LogSettings::new_default_settings()).unwrap()
    );
  }
}
