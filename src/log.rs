use colored::*;

pub fn log_info(message: String) {
    println!("{}", message.blue());
}

pub fn log_warning(message: String) {
    println!("{}", message.yellow());
}

pub fn log_error(message: String) {
    println!("{}", message.red());
}
