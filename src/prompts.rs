use std::env;
use std::fs;

/// Build system and user prompts for the `fix` subcommand.
pub fn fix_prompt(
    command: &str,
    exit_code: i32,
    shell: &str,
    os_context: &str,
    history: &str,
) -> (String, String) {
    let system = "You are a shell command expert. The user ran a command that failed. \
        Suggest the corrected command. Reply with EXACTLY two lines: the first line is \
        ONLY the corrected command (no backticks, no explanation), the second line is a \
        brief explanation of what went wrong and what the fix does."
        .to_string();

    let mut user = format!(
        "Failed command: {}\nExit code: {}\nShell: {}\n{}",
        command, exit_code, shell, os_context
    );

    if !history.is_empty() {
        user.push_str(&format!("\n\nRecent history:\n{}", history));
    }

    (system, user)
}

/// Build system and user prompts for the `explain` subcommand.
pub fn explain_prompt(command: &str, os_context: &str) -> (String, String) {
    let system = "You are a shell command expert. Explain what the given command does \
        concisely. Break down each flag and argument."
        .to_string();

    let user = format!("Command: {}\n\n{}", command, os_context);

    (system, user)
}

/// Build system and user prompts for the `suggest` subcommand.
pub fn suggest_prompt(description: &str, os_context: &str) -> (String, String) {
    let system = "You are a shell command expert. Suggest a command that accomplishes \
        the user's goal. Reply with EXACTLY two lines: the first line is ONLY the command, \
        the second line is a brief explanation."
        .to_string();

    let user = format!("{}\n\n{}", description, os_context);

    (system, user)
}

/// Build system and user prompts for the `ask` subcommand.
pub fn ask_prompt(question: &str, os_context: &str) -> (String, String) {
    let system = "You are a knowledgeable shell and Linux expert. Answer the user's \
        question concisely, providing commands when relevant."
        .to_string();

    let user = format!("{}\n\n{}", question, os_context);

    (system, user)
}

/// Gather OS and environment context for prompt enrichment.
pub fn get_os_context() -> String {
    let pretty_name = read_os_pretty_name().unwrap_or_else(|| {
        // Fallback to uname
        std::process::Command::new("uname")
            .arg("-s")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    });

    let shell = env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|n| n.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let arch = std::process::Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!("OS: {}, Shell: {}, Arch: {}", pretty_name, shell, arch)
}

fn read_os_pretty_name() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}
