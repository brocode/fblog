use lazy_static::lazy_static;
use std::env;
use yansi::{Color, Style};

lazy_static! {
  static ref NO_COLOR: bool = env::var("NO_COLOR").is_ok();
}

pub fn paint(c: Color, t: &str) -> String {
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

pub fn stylew<W: std::fmt::Write>(mut target: W, s: &Style, t: &str) {
  if *NO_COLOR {
    _ = target.write_str(t);
  } else {
    _ = write!(target, "{}", s.paint(t));
  }
}
