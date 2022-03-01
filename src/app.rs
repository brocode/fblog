use crate::template;
use clap::crate_version;
use clap::{Arg, Command};

pub fn app<'a>() -> Command<'a> {
  Command::new("fblog")
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(
      Arg::new("additional-value")
        .long("additional-value")
        .short('a')
        .multiple_occurrences(true)
        .number_of_values(1)
        .takes_value(true)
        .conflicts_with("excluded-value")
        .help("adds additional values"),
    )
    .arg(
      Arg::new("message-key")
        .long("message-key")
        .short('m')
        .multiple_occurrences(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the message in the log entry. The first matching key will be assigned to `fblog_message`."),
    )
    .arg(
      Arg::new("print-lua")
        .long("print-lua")
        .multiple_occurrences(false)
        .takes_value(false)
        .help("Prints lua init expressions. Used for fblog debugging."),
    )
    .arg(
      Arg::new("time-key")
        .long("time-key")
        .short('t')
        .multiple_occurrences(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the time in the log entry. The first matching key will be assigned to `fblog_timestamp`."),
    )
    .arg(
      Arg::new("level-key")
        .long("level-key")
        .short('l')
        .multiple_occurrences(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the level in the log entry. The first matching key will be assigned to `fblog_level`."),
    )
    .arg(
      Arg::new("dump-all")
        .long("dump-all")
        .short('d')
        .multiple_occurrences(false)
        .takes_value(false)
        .help("dumps all values"),
    )
    .arg(
      Arg::new("excluded-value")
        .long("excluded-value")
        .short('x')
        .multiple_occurrences(true)
        .number_of_values(1)
        .takes_value(true)
        .conflicts_with("additional-value")
        .help("Excludes values (--dump-all is enabled implicitly)"),
    )
    .arg(
      Arg::new("with-prefix")
        .long("with-prefix")
        .short('p')
        .multiple_occurrences(false)
        .takes_value(false)
        .help("consider all text before opening curly brace as prefix"),
    )
    .arg(
      Arg::new("filter")
        .long("filter")
        .short('f')
        .multiple_occurrences(false)
        .takes_value(true)
        .help("lua expression to filter log entries. `message ~= nil and string.find(message, \"text.*\") ~= nil`"),
    )
    .arg(
      Arg::new("no-implicit-filter-return-statement")
        .long("no-implicit-filter-return-statement")
        .multiple_occurrences(false)
        .takes_value(false)
        .help("if you pass a filter expression 'return' is automatically prepended. Pass this switch to disable the implicit return."),
    )
    .arg(
      Arg::new("INPUT")
        .help("Sets the input file to use, otherwise assumes stdin")
        .required(false)
        .default_value("-"),
    )
    .arg(
      Arg::new("main-line-format")
        .long("main-line-format")
        .number_of_values(1)
        .takes_value(true)
        .default_value(template::DEFAULT_MAIN_LINE_FORMAT)
        .help("Formats the main fblog output. All log values can be used. fblog provides sanitized variables starting with `fblog_`."),
    )
    .arg(
      Arg::new("additional-value-format")
        .long("additional-value-format")
        .number_of_values(1)
        .takes_value(true)
        .default_value(template::DEFAULT_ADDITIONAL_VALUE_FORMAT)
        .help("Formats the additional value fblog output."),
    )
}
