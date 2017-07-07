use clap::{App, AppSettings, Arg};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead};

#[macro_use]
extern crate clap;

extern crate serde_json;
extern crate ansi_term;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(test)]
extern crate regex;

mod log;

fn main() {
  let app = app();
  let matches = app.get_matches();
  let additional_values: Vec<String> = matches.values_of("additional-value")
                                              .map(|values| values.map(|v| v.to_owned()).collect())
                                              .unwrap_or_default();

  let dump_all = matches.is_present("dump-all");

  let stdin = io::stdin();
  let reader = stdin.lock();
  for line in reader.lines() {
    let read_line = &line.expect("Should be able to read line");
    match serde_json::from_str::<Value>(read_line) {
      Ok(Value::Object(log_entry)) => log::print_log_line(&mut io::stdout(), &extract_string_values(&log_entry), &additional_values, dump_all),
      _ => println!("??? > {}", read_line),
    };
  }
}

fn app<'a>() -> App<'a, 'a> {
  App::new("fblog")
    .setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(Arg::with_name("additional-value")
           .short("a")
           .multiple(true)
           .takes_value(true)
           .help("adds additional values"))
    .arg(Arg::with_name("dump-all")
           .short("d")
           .multiple(false)
           .takes_value(false)
           .help("dumps all values"))
}

fn extract_string_values(log_entry: &Map<String, Value>) -> BTreeMap<String, String> {
  let mut string_map: BTreeMap<String, String> = BTreeMap::new();
  for (key, value) in log_entry {
    if let Value::String(ref string_value) = *value {
      string_map.insert(key.to_owned(), string_value.to_owned());
    }
  }
  string_map
}

