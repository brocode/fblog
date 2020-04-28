use crate::template;
use clap::crate_version;
use clap::{App, AppSettings, Arg};

pub fn app<'a>() -> App<'a, 'a> {
  App::new("fblog")
    .global_setting(AppSettings::ColoredHelp)
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(
      Arg::with_name("additional-value")
        .long("additional-value")
        .short("a")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("adds additional values"),
    )
    .arg(
      Arg::with_name("message-key")
        .long("message-key")
        .short("m")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the message in the log entry."),
    )
    .arg(
      Arg::with_name("time-key")
        .long("time-key")
        .short("t")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the time in the log entry."),
    )
    .arg(
      Arg::with_name("level-key")
        .long("level-key")
        .short("l")
        .multiple(true)
        .number_of_values(1)
        .takes_value(true)
        .help("Adds an additional key to detect the level in the log entry."),
    )
    .arg(
      Arg::with_name("dump-all")
        .long("dump-all")
        .short("d")
        .multiple(false)
        .takes_value(false)
        .help("dumps all values"),
    )
    .arg(
      Arg::with_name("with-prefix")
        .long("with-prefix")
        .short("p")
        .multiple(false)
        .takes_value(false)
        .help("consider all text before opening curly brace as prefix"),
    )
    .arg(
      Arg::with_name("filter")
        .long("filter")
        .short("f")
        .multiple(false)
        .takes_value(true)
        .help("lua expression to filter log entries. `message ~= nil and string.find(message, \"text.*\") ~= nil`"),
    )
    .arg(
      Arg::with_name("no-implicit-filter-return-statement")
        .long("no-implicit-filter-return-statement")
        .multiple(false)
        .takes_value(false)
        .help("if you pass a filter expression 'return' is automatically prepended. Pass this switch to disable the implicit return."),
    )
    .arg(
      Arg::with_name("INPUT")
        .help("Sets the input file to use, otherwise assumes stdin")
        .required(false)
        .default_value("-"),
    )
    .arg(
      Arg::with_name("inspect")
        .long("inspect")
        .short("i")
        .multiple(false)
        .takes_value(false)
        .help("only prints json keys not encountered before"),
    )
    .arg(
      Arg::with_name("main-line-format")
        .long("main-line-format")
        .number_of_values(1)
        .takes_value(true)
        .default_value(template::DEFAULT_MAIN_LINE_FORMAT)
        .help("Formats the main fblog output. All log values can be used. fblog provides sanitized variables starting with `fblog_`."),
    )
    .arg(
      Arg::with_name("additional-value-format")
        .long("additional-value-format")
        .number_of_values(1)
        .takes_value(true)
        .default_value(template::DEFAULT_ADDITIONAL_VALUE_FORMAT)
        .help("Formats the addtional value fblog output."),
    )
}
