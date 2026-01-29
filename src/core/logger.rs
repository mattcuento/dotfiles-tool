use colored::Colorize;

pub fn log_info(msg: &str) {
    println!("{} {}", "[INFO]".blue(), msg);
}

pub fn log_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

pub fn log_warn(msg: &str) {
    println!("{} {}", "⚠".yellow(), msg);
}

pub fn log_error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}
