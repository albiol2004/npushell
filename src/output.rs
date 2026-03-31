use std::io::{self, BufRead, Write};

// ANSI escape codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

/// Wrap text with an ANSI color code.
pub fn colored(text: &str, color: &str) -> String {
    let code = match color {
        "green" => GREEN,
        "yellow" => YELLOW,
        "cyan" => CYAN,
        _ => "",
    };
    format!("{}{}{}", code, text, RESET)
}

/// Wrap text in bold.
pub fn bold(text: &str) -> String {
    format!("{}{}{}", BOLD, text, RESET)
}

/// Wrap text in dim.
pub fn dim(text: &str) -> String {
    format!("{}{}{}", DIM, text, RESET)
}

/// Display a command suggestion with optional explanation.
pub fn print_suggestion(command: &str, explanation: &str, use_color: bool) {
    if use_color {
        println!(
            "\n{}  {}\n",
            bold(&colored("Suggestion:", "green")),
            bold(&colored(command, "cyan"))
        );
        if !explanation.is_empty() {
            println!("  {}", dim(explanation));
        }
    } else {
        println!("\nSuggestion: {}\n", command);
        if !explanation.is_empty() {
            println!("  {}", explanation);
        }
    }
}

/// Display an explanation of a command.
pub fn print_explanation(text: &str, use_color: bool) {
    if use_color {
        println!("\n{}\n", bold(&colored("Explanation:", "yellow")));
        println!("{}\n", text);
    } else {
        println!("\nExplanation:\n");
        println!("{}\n", text);
    }
}

/// Display a general response.
pub fn print_response(text: &str, use_color: bool) {
    if use_color {
        println!("\n{}\n", colored(text, "cyan"));
    } else {
        println!("\n{}\n", text);
    }
}

/// Prompt the user for yes/no confirmation. Returns true if the user confirms.
#[allow(dead_code)]
pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        let answer = line.trim().to_lowercase();
        matches!(answer.as_str(), "y" | "yes")
    } else {
        false
    }
}
