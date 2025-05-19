use colored::Colorize;
use std::fmt::Display;

pub fn print_success<T: Display>(message: T) {
    println!("{} {}", "✔".green(), message)
}

pub fn print_warning<T: Display>(message: T) {
    println!("{} {}", "⚠".yellow(), message)
}

pub fn print_error<T: Display>(message: T) {
    eprintln!("{} {}", "⨯".red(), message.to_string().red())
}

pub fn print_info<T: Display>(message: T) {
    // println!("{} {}", "ℹ".blue(), message)
    println!("{} {}", "".blue(), message)
}
