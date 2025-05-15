use handlebars::{Handlebars, handlebars_helper, no_escape};
use std::convert::TryInto;
use yansi::{Color, Paint};

pub static DEFAULT_MAIN_LINE_FORMAT: &str = "{{bold(fixed_size 19 fblog_timestamp)}} {{level_style (uppercase (fixed_size 5 fblog_level))}}:{{#if fblog_prefix}} {{bold(cyan fblog_prefix)}}{{/if}} {{fblog_message}}";
pub static DEFAULT_ADDITIONAL_VALUE_FORMAT: &str = "{{bold (color_rgb 150 150 150 (min_size 25 key))}}: {{value}}";

pub fn fblog_handlebar_registry(main_line_format: String, additional_value_format: String) -> Handlebars<'static> {
	handlebars_helper!(bold: |t: str| {
			format!("{}", t.bold())
	});

	handlebars_helper!(cyan: |t: str| {
			format!("{}", t.cyan())
	});

	handlebars_helper!(yellow: |t: str| {
			format!("{}", t.yellow())
	});

	handlebars_helper!(red: |t: str| {
			format!("{}", t.red())
	});

	handlebars_helper!(blue: |t: str| {
			format!("{}", t.blue())
	});

	handlebars_helper!(purple: |t: str| {
			format!("{}", t.magenta())
	});

	handlebars_helper!(green: |t: str| {
			format!("{}", t.green())
	});

	handlebars_helper!(color_rgb: |r: u64, g: u64, b: u64, t: str| {
			format!("{}", t.rgb(r.try_into().unwrap(), g.try_into().unwrap(), b.try_into().unwrap()))
	});

	handlebars_helper!(uppercase: |t: str| {
			t.to_uppercase()
	});

	handlebars_helper!(level_style: |level: str| {
			let color = match level.trim().to_lowercase().as_ref() {
					"trace" => Color::Cyan,
					"debug" => Color::Blue,
					"info" => Color::Green,
					"warn" | "warning" => Color::Yellow,
					"error" | "err" => Color::Red,
					"fatal" => Color::Magenta,
					_ => Color::Magenta,
			};
			format!("{}", level.fg(color).bold())
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

	handlebars_helper!(min_size: |isize: u64, t: str| {
			let x = t.to_string();
			let size = isize.try_into().expect("should fit");
			if x.len() < size {
				 format!("{}{}", " ".repeat(size - x.len()), x)
			} else {
				x
			}
	});

	let mut reg = Handlebars::new();
	reg.register_escape_fn(Box::new(no_escape));

	reg.register_helper("bold", Box::new(bold));
	reg.register_helper("uppercase", Box::new(uppercase));
	reg.register_helper("fixed_size", Box::new(fixed_size));
	reg.register_helper("min_size", Box::new(min_size));
	reg.register_helper("level_style", Box::new(level_style));

	reg.register_helper("yellow", Box::new(yellow));
	reg.register_helper("cyan", Box::new(cyan));
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
