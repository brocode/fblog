use crate::no_color_support::{paint, style};
use ansi_term::{Colour, Style};
use handlebars::{handlebars_helper, Handlebars};
use std::convert::TryInto;

pub static DEFAULT_MAIN_LINE_FORMAT: &str = "{{bold(fixed_size 19 fblog_timestamp)}} {{level_style (uppercase (fixed_size 5 fblog_level))}}:{{bold(color_rgb 138 43 226 fblog_prefix)}} {{fblog_message}}";
pub static DEFAULT_ADDITIONAL_VALUE_FORMAT: &str = "{{bold (color_rgb 150 150 150 (fixed_size 25 key))}}: {{value}}";

fn level_to_style(level: &str) -> Style {
  match level.trim().to_lowercase().as_ref() {
    "info" => Colour::Green,
    "warn" | "warning" => Colour::Yellow,
    "error" | "err" => Colour::Red,
    "debug" => Colour::Blue,
    _ => Colour::Purple,
  }
  .bold()
}

pub fn fblog_handlebar_registry(main_line_format: String, additional_value_format: String) -> Handlebars<'static> {
  handlebars_helper!(bold: |t: str| {
      style(&Style::new().bold(), t)
  });

  handlebars_helper!(yellow: |t: str| {
      paint(Colour::Yellow, t)
  });

  handlebars_helper!(red: |t: str| {
      paint(Colour::Red, t)
  });

  handlebars_helper!(blue: |t: str| {
      paint(Colour::Blue, t)
  });

  handlebars_helper!(purple: |t: str| {
      paint(Colour::Purple, t)
  });

  handlebars_helper!(green: |t: str| {
      paint(Colour::Green, t)
  });

  handlebars_helper!(color_rgb: |r: u64, g: u64, b: u64, t: str| {
      paint(Colour::RGB(r.try_into().unwrap(), g.try_into().unwrap(), b.try_into().unwrap()), t)
  });

  handlebars_helper!(uppercase: |t: str| {
      t.to_uppercase()
  });

  handlebars_helper!(level_style: |level: str| {
      let s = level_to_style(level);
      style(&s, level)
  });

  handlebars_helper!(fixed_size: |isize: u64, t: str| {
      let mut x = t.to_string();
      let size = isize.try_into().expect("should fit");
      x.truncate(size);
      if x.len() < size {
         format!("{}{}", " ".repeat(size - x.len()), x)
      } else {
        x
      }
  });

  let mut reg = Handlebars::new();

  reg.register_helper("bold", Box::new(bold));
  reg.register_helper("uppercase", Box::new(uppercase));
  reg.register_helper("fixed_size", Box::new(fixed_size));
  reg.register_helper("level_style", Box::new(level_style));

  reg.register_helper("yellow", Box::new(yellow));
  reg.register_helper("red", Box::new(red));
  reg.register_helper("blue", Box::new(blue));
  reg.register_helper("purple", Box::new(purple));
  reg.register_helper("green", Box::new(green));
  reg.register_helper("color_rgb", Box::new(color_rgb));

  reg.register_template_string("main_line", main_line_format).expect("Template invalid");
  reg
    .register_template_string("additional_value", additional_value_format)
    .expect("Template invalid");
  reg
}
