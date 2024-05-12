use std::io::Write;

use clap::builder::styling::{AnsiColor, Color, Reset, Style};

pub struct LoggerStyles {
    success: Style,
    info: Style,
    error: Style,
    debug: Style,

    prompt: Style,
    tag: Style,
}

#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug = 0,
    Info,
    Success,
    Error,
    // Off,
}

pub struct Logger {
    styles: LoggerStyles,
    level: LogLevel,
}

impl Logger {
    pub fn new(styles: Option<LoggerStyles>, level: Option<LogLevel>) -> Self {
        Self {
            styles: styles.unwrap_or(LoggerStyles {
                success: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
                info: Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))),
                error: Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
                debug: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlue))),
                prompt: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightWhite))),

                tag: Style::new().bold(),
            }),
            level: level.unwrap_or(LogLevel::Info),
        }
    }

    pub fn success(&self, message: std::fmt::Arguments) {
        self.log(
            LogLevel::Success,
            format_args!(
                "{}[{}OK{:#}{}] {}{:#}",
                self.styles.success, self.styles.tag, Reset, self.styles.success, message, Reset,
            ),
        )
    }

    pub fn info(&self, message: std::fmt::Arguments) {
        self.log(
            LogLevel::Info,
            format_args!(
                "{}[{}INFO{:#}{}] {}{:#}",
                self.styles.info, self.styles.tag, Reset, self.styles.info, message, Reset,
            ),
        )
    }

    pub fn error(&self, message: std::fmt::Arguments) {
        self.log(
            LogLevel::Error,
            format_args!(
                "{}[{}ERROR{:#}{}] {}{:#}",
                self.styles.error, self.styles.tag, Reset, self.styles.error, message, Reset,
            ),
        )
    }

    pub fn debug(&self, message: std::fmt::Arguments) {
        self.log(
            LogLevel::Debug,
            format_args!(
                "{}[{}DEBUG{:#}{}] {}{:#}",
                self.styles.debug, self.styles.tag, Reset, self.styles.debug, message, Reset,
            ),
        )
    }

    pub fn prompt_char(&self, message: std::fmt::Arguments) -> std::io::Result<Option<char>> {
        print!(
            "{}[{}PROMPT{:#}{}] {}{:#} ",
            self.styles.prompt, self.styles.tag, Reset, self.styles.prompt, message, Reset,
        );
        let mut input_buf = String::new();
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input_buf)?;

        Ok(input_buf.chars().next())
    }

    fn log(&self, level: LogLevel, message: std::fmt::Arguments) {
        if self.level <= level {
            println!("{}", message);
        }
    }
}

impl Default for Logger {
    // TODO: better log management, shouldn't create new logger everywhere
    fn default() -> Self {
        Self::new(None, None)
    }
}
