// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#![allow(unused)]

use colored::{Color, Colorize};
use terminal_size::{Width, terminal_size};
use textwrap::{Options, fill};

#[derive(Clone, Copy)]
pub enum LoggerColor {
    Red,
    Yellow,
    Green,
    Blue,
    Cyan,
    Magenta,
    White,
    Black,
}

impl LoggerColor {
    fn into_colored(self) -> Color {
        match self {
            LoggerColor::Red => Color::BrightRed,
            LoggerColor::Yellow => Color::BrightYellow,
            LoggerColor::Green => Color::BrightGreen,
            LoggerColor::Blue => Color::BrightBlue,
            LoggerColor::Cyan => Color::BrightCyan,
            LoggerColor::Magenta => Color::BrightMagenta,
            LoggerColor::White => Color::White,
            LoggerColor::Black => Color::Black,
        }
    }
}

pub struct Logger {
    width: usize,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        colored::control::set_override(true);

        let width = terminal_size()
            .map(|(Width(w), _)| (w as usize).min(140))
            .unwrap_or(140);

        Self { width }
    }

    pub fn with_width(width: usize) -> Self {
        Self { width }
    }

    pub fn error_log(&self, msg: &str) {
        self.print_wrapped("error", LoggerColor::Red, msg);
    }

    pub fn warning_log(&self, msg: &str) {
        self.print_wrapped("warning", LoggerColor::Yellow, msg);
    }

    pub fn info_log(&self, msg: &str) {
        self.print_wrapped("info", LoggerColor::Cyan, msg);
    }

    pub fn custom_log(&self, prefix: &str, color: LoggerColor, msg: &str) {
        self.print_wrapped(prefix, color, msg);
    }

    fn print_wrapped(&self, prefix: &str, color: LoggerColor, msg: &str) {
        let prefix_colored = prefix.color(color.into_colored()).bold();
        let indent_len = prefix.chars().count() + 2;
        let indent = " ".repeat(indent_len);

        let options = Options::new(self.width)
            .initial_indent("")
            .subsequent_indent(&indent);

        let wrapped = fill(msg, options);
        let mut lines = wrapped.lines();

        if let Some(first) = lines.next() {
            println!("{}: {}", prefix_colored, first.white().bold());
        }

        for line in lines {
            println!("{}", line.white().bold());
        }
    }

    pub fn code_snippet(&self, file: &str, line: usize, col: usize, code: &str, msg: &str) {
        let line_str = line.to_string();
        let gutter_space = " ".repeat(line_str.len());
        let blue = Color::BrightBlue;

        println!(
            "{} {} {}:{}:{}",
            gutter_space,
            "-->".color(blue).bold(),
            file,
            line,
            col
        );
        println!("{} {}", gutter_space, "|".color(blue).bold());
        println!(
            "{} {} {}",
            line_str.color(blue).bold(),
            "|".color(blue).bold(),
            code
        );

        let pointer_indent = " ".repeat(col.saturating_sub(1));
        let carets_len = code
            .chars()
            .count()
            .saturating_sub(col.saturating_sub(1))
            .max(1);
        let carets = "^".repeat(carets_len);

        println!(
            "{} {} {}{} {}",
            gutter_space,
            "|".color(blue).bold(),
            pointer_indent,
            carets.color(Color::BrightRed).bold(),
            msg.color(Color::BrightRed).bold()
        );
        println!("{} {}", gutter_space, "|".color(blue).bold());
    }

    pub fn code_add(&self, code: &str) {
        println!(
            "{} {} {}",
            "+".color(Color::BrightGreen).bold(),
            "|".color(Color::BrightBlue).bold(),
            code.color(Color::BrightGreen)
        );
    }

    pub fn code_remove(&self, code: &str) {
        println!(
            "{} {} {}",
            "-".color(Color::BrightRed).bold(),
            "|".color(Color::BrightBlue).bold(),
            code.color(Color::BrightRed)
        );
    }
}
