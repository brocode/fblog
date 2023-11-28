use crate::substitution::Substitution;
use clap::{crate_version, value_parser, ArgAction, ValueHint};
use clap::{Arg, Command};
use clap_complete::Shell;

pub fn app() -> Command {
    Command::new("fblog")
    .version(crate_version!())
    .author("Brocode inc <bros@brocode.sh>")
    .about("json log viewer")
    .arg(
      Arg::new("additional-value")
        .long("additional-value")
        .short('a')
        .action(ArgAction::Append)
        .num_args(1)
        .conflicts_with("excluded-value")
        .help("adds additional values"),
    )
    .arg(
      Arg::new("config-file")
        .long("config-file")
        .action(ArgAction::Set)
        .num_args(1)
        .help("configuration file to load"),
    )
    .arg(
      Arg::new("generate-completions")
        .long("generate-completions")
        .action(ArgAction::Set)
        .hide(true)
        .value_parser(value_parser!(Shell)),
    )
    .arg(
      Arg::new("message-key")
        .long("message-key")
        .short('m')
        .action(ArgAction::Append)
        .num_args(1)
        .help("Adds an additional key to detect the message in the log entry. The first matching key will be assigned to `fblog_message`."),
    )
    .arg(
      Arg::new("print-lua")
        .long("print-lua")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .help("Prints lua init expressions. Used for fblog debugging."),
    )
    .arg(
      Arg::new("time-key")
        .long("time-key")
        .short('t')
        .action(ArgAction::Append)
        .num_args(1)
        .help("Adds an additional key to detect the time in the log entry. The first matching key will be assigned to `fblog_timestamp`."),
    )
    .arg(
      Arg::new("level-key")
        .long("level-key")
        .short('l')
        .action(ArgAction::Append)
        .num_args(1)
        .help("Adds an additional key to detect the level in the log entry. The first matching key will be assigned to `fblog_level`."),
    )
    .arg(
      Arg::new("dump-all")
        .long("dump-all")
        .short('d')
        .num_args(0)
        .action(ArgAction::SetTrue)
        .help("dumps all values"),
    )
    .arg(
      Arg::new("excluded-value")
        .long("excluded-value")
        .short('x')
        .action(ArgAction::Append)
        .num_args(1)
        .conflicts_with("additional-value")
        .help("Excludes values (--dump-all is enabled implicitly)"),
    )
    .arg(
      Arg::new("with-prefix")
        .long("with-prefix")
        .short('p')
        .num_args(0)
        .action(ArgAction::SetTrue)
        .help("consider all text before opening curly brace as prefix"),
    )
    .arg(
      Arg::new("filter")
        .long("filter")
        .short('f')
        .action(ArgAction::Set)
        .num_args(1)
        .help("lua expression to filter log entries. `message ~= nil and string.find(message, \"text.*\") ~= nil`"),
    )
    .arg(
      Arg::new("no-implicit-filter-return-statement")
        .long("no-implicit-filter-return-statement")
        .num_args(0)
        .action(ArgAction::SetTrue)
        .help("if you pass a filter expression 'return' is automatically prepended. Pass this switch to disable the implicit return."),
    )
    .arg(
      Arg::new("INPUT")
        .help("Sets the input file to use, otherwise assumes stdin")
        .required(false)
        .value_hint(ValueHint::AnyPath)
        .default_value("-"),
    )
    .arg(
      Arg::new("main-line-format")
        .long("main-line-format")
        .num_args(1)
        .help("Formats the main fblog output. All log values can be used. fblog provides sanitized variables starting with `fblog_`."),
    )
    .arg(
      Arg::new("additional-value-format")
        .long("additional-value-format")
        .num_args(1)
        .help("Formats the additional value fblog output."),
    )
    .arg(
      Arg::new("enable-substitution")
        .long("substitute")
        .short('s')
        .action(ArgAction::SetTrue)
        .help("Enable substitution of placeholders in the log messages with their corresponding values from the context."),
    )
    .arg(
      Arg::new("context-key")
        .long("context-key")
        .short('c')
        .num_args(1)
        .action(ArgAction::Set)
        .default_value_if("enable-substitution", "true", Substitution::DEFAULT_CONTEXT_KEY)
        .help("Use this key as the source of substitutions for the message. Value can either be an array ({1}) or an object ({key})."),
    )
    .arg(
      Arg::new("placeholder-format")
        .long("placeholder-format")
        .short('F')
        .num_args(1)
        .action(ArgAction::Set)
        .default_value_if("enable-substitution", "true", Substitution::DEFAULT_PLACEHOLDER_FORMAT)
        .help("The format that should be used for substituting values in the message, where the key is the literal word `key`. Example: [[key]] or ${key}."),
    )
}
