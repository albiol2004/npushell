use std::env;
use std::fs;
use std::path::PathBuf;

/// Read the most recent lines from the user's shell history file.
///
/// Tries `$HISTFILE` first, then `~/.bash_history`, then `~/.zsh_history`.
/// Returns an empty string on any error (history is non-critical context).
pub fn read_recent_history(max_lines: usize) -> String {
    let candidates = history_file_candidates();

    for path in candidates {
        if let Ok(contents) = fs::read_to_string(&path) {
            let lines: Vec<&str> = contents.lines().collect();
            let start = lines.len().saturating_sub(max_lines);
            return lines[start..].join("\n");
        }
    }

    String::new()
}

fn history_file_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(histfile) = env::var("HISTFILE") {
        candidates.push(PathBuf::from(histfile));
    }

    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".bash_history"));
        candidates.push(home.join(".zsh_history"));
    }

    candidates
}
