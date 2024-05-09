use clap::builder::styling::{AnsiColor, Color, Style};

pub struct LoggerStyles {
    success: Style,
    info: Style,
}

pub struct Logger {
    styles: LoggerStyles,
}

impl Logger {
    pub fn new(styles: Option<LoggerStyles>) -> Self {
        Self {
            styles: styles.unwrap_or(LoggerStyles {
                success: Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
                info: Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))),
            }),
        }
    }

    pub fn success(&self, message: &str) {
        println!(
            "{}[ OK ] {}{:#}",
            self.styles.success,
            message,
            self.styles.success,
        );
    }

    pub fn info(&self, message: &str) {
        println!(
            "{}[INFO] {}{:#}",
            self.styles.info,
            message,
            self.styles.info,
        );
    }
}

