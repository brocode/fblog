use ansi_term::{Colour, Style};
use lazy_static::lazy_static;
use std::env;

lazy_static! {
  static ref NO_COLOR: bool = env::var("NO_COLOR").is_ok();
}

pub fn paint(c: Colour, t: &str) -> String {
  if *NO_COLOR {
    t.to_string()
  } else {
    format!("{}", c.paint(t))
  }
}

pub fn style(s: &Style, t: &str) -> String {
  if *NO_COLOR {
    t.to_string()
  } else {
    format!("{}", s.paint(t))
  }
}
