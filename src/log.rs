use clap::builder::styling::{AnsiColor, Reset};

pub fn success(message: &str) {
    println!("{}[ OK ] {}{}", AnsiColor::Green.render_fg(), message, Reset.render());
}

pub fn info(message: &str) {
    println!("{}[INFO] {}{}", AnsiColor::White.render_fg(), message, Reset.render());
}
