use colored::{Color, Colorize};
use log::warn;

use crate::translate::Token;

enum LogType {
    Info,
    Warning,
    Error,
}

fn format_token(token: &Token, log_type: LogType) -> String {
    let highlight_symbol = match log_type {
        LogType::Info => "﹉",
        LogType::Warning => "~",
        LogType::Error => "‾",
    };

    let color = match log_type {
        LogType::Info => Color::Green,
        LogType::Warning => Color::Yellow,
        LogType::Error => Color::Red,
    };

    format!(
        "{} | {}\n{}   {}{}",
        token.line.to_string(),
        token.line_contents,
        " ".repeat(token.line.to_string().len()),
        " ".repeat(token.start),
        highlight_symbol
            .repeat(token.end - token.start)
            .color(color)
    )
}

pub fn warn_token(token: &Token, message: &str) {
    println!("{}", format_token(token, LogType::Warning));
    warn!("{}\n", message);
}
