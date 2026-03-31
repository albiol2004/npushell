mod client;
mod config;
mod history;
#[allow(dead_code)]
mod output;
mod prompts;

use clap::{Parser, Subcommand};

use client::CopilotClient;
use config::Config;
use history::read_recent_history;
use prompts::{ask_prompt, explain_prompt, fix_prompt, get_os_context, suggest_prompt};

#[derive(Parser)]
#[command(
    name = "npushell",
    version,
    about = "NPU-powered shell copilot",
    arg_required_else_help = false,
    infer_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Free-form query (when no subcommand is used)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    query: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a failed command and suggest a fix (called by shell hooks)
    Fix {
        /// The command that failed
        #[arg(long)]
        command: String,

        /// The exit code of the failed command
        #[arg(long)]
        exit_code: i32,

        /// The current shell (bash, zsh, etc.)
        #[arg(long)]
        shell: String,

        /// PID of the shell process
        #[arg(long)]
        pid: u32,
    },

    /// Explain what a command does
    Explain {
        /// The command to explain
        command: String,
    },

    /// Suggest a command from a natural language description
    Suggest {
        /// What you want to accomplish
        description: String,
    },

    /// Ask a free-form question
    Ask {
        /// Your question
        question: String,
    },

    /// Check API connectivity and hook installation
    Doctor,

    /// Show current configuration
    Config,

    /// Set up npushell: create config and install shell hooks
    Init,
}

fn main() {
    let cli = Cli::parse();
    let config = Config::load();
    let client = CopilotClient::new(&config.api);

    match cli.command {
        Some(Commands::Fix {
            command,
            exit_code,
            shell,
            pid,
        }) => {
            handle_fix(&config, &client, &command, exit_code, &shell, pid);
        }

        Some(Commands::Explain { command }) => {
            handle_explain(&config, &client, &command);
        }

        Some(Commands::Suggest { description }) => {
            handle_suggest(&config, &client, &description);
        }

        Some(Commands::Ask { question }) => {
            handle_ask(&config, &client, &question);
        }

        Some(Commands::Doctor) => {
            handle_doctor(&config, &client);
        }

        Some(Commands::Config) => {
            config.display();
        }

        Some(Commands::Init) => {
            handle_init(&config, &client);
        }

        None => {
            if cli.query.is_empty() {
                // No subcommand and no query: print help
                use clap::CommandFactory;
                Cli::command().print_help().ok();
                println!();
            } else {
                // Check if the first word looks like a misspelled subcommand
                let first = &cli.query[0];
                let subcommands = ["fix", "explain", "suggest", "ask", "doctor", "config", "init"];
                if let Some(suggestion) = find_similar(first, &subcommands) {
                    eprintln!(
                        "Unknown subcommand '{}'. Did you mean '{}'?\n",
                        first, suggestion
                    );
                    eprintln!("To ask a question, use: npushell ask \"{}\"", cli.query.join(" "));
                    std::process::exit(1);
                }
                // Implicit ask mode
                let question = cli.query.join(" ");
                handle_ask(&config, &client, &question);
            }
        }
    }
}

/// Strip markdown formatting from LLM output (backticks, code blocks)
fn strip_markdown(text: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            result.push_str(line);
        } else {
            // Strip inline backticks
            result.push_str(&line.replace('`', ""));
        }
        result.push('\n');
    }

    result.trim().to_string()
}

fn handle_fix(
    _config: &Config,
    client: &CopilotClient,
    command: &str,
    exit_code: i32,
    shell: &str,
    pid: u32,
) {
    let history = read_recent_history(_config.behavior.max_history_context);
    let os_ctx = get_os_context();
    let (sys, usr) = fix_prompt(command, exit_code, shell, &os_ctx, &history);

    if let Ok(response) = client.complete(&sys, &usr, 150) {
        let cleaned = strip_markdown(&response);
        let mut lines = cleaned.lines();
        let cmd = lines.next().unwrap_or("").to_string();
        let explanation = lines.collect::<Vec<_>>().join("\n");

        // Write suggestion file (for precmd fallback)
        let path = format!("/tmp/npushell-suggestion.{}", pid);
        let content = format!("COMMAND:{}\nEXPLANATION:{}", cmd, explanation);
        std::fs::write(&path, content).ok();

        // Also print directly to /dev/tty so it appears immediately,
        // even if the user hasn't pressed Enter yet
        if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
            use std::io::Write;
            let _ = write!(tty, "\r\n");
            let _ = write!(
                tty,
                "\x1b[1;36m npushell\x1b[0m \x1b[2m\u{2500} suggested fix:\x1b[0m\r\n"
            );
            let _ = write!(tty, "  \x1b[1;32m$ {}\x1b[0m\r\n", cmd);
            if !explanation.is_empty() {
                let _ = write!(tty, "  \x1b[2m{}\x1b[0m\r\n", explanation);
            }
            let _ = write!(
                tty,
                "\x1b[2m  (press Enter to see prompt)\x1b[0m\r\n"
            );
        }
    }
}

fn handle_explain(config: &Config, client: &CopilotClient, command: &str) {
    let os_ctx = get_os_context();
    let (sys, usr) = explain_prompt(command, &os_ctx);

    if config.ui.color {
        println!("\n{}\n", output::bold(&output::colored("Explanation:", "yellow")));
    } else {
        println!("\nExplanation:\n");
    }

    match client.complete_streaming(&sys, &usr, 512) {
        Ok(_) => println!(),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn handle_suggest(config: &Config, client: &CopilotClient, description: &str) {
    let os_ctx = get_os_context();
    let (sys, usr) = suggest_prompt(description, &os_ctx);

    if config.ui.color {
        print!("\n{} ", output::bold(&output::colored("Suggestion:", "green")));
    } else {
        print!("\nSuggestion: ");
    }

    match client.complete_streaming(&sys, &usr, 200) {
        Ok(_) => println!(),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn handle_ask(_config: &Config, client: &CopilotClient, question: &str) {
    let os_ctx = get_os_context();
    let (sys, usr) = ask_prompt(question, &os_ctx);

    println!();
    match client.complete_streaming(&sys, &usr, 1024) {
        Ok(_) => println!(),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn handle_init(config: &Config, client: &CopilotClient) {
    use std::io::{self, BufRead, Write};

    println!("npushell init");
    println!("=============\n");

    // 1. Create config file if it doesn't exist
    let config_path = Config::config_path();
    print!("Config file... ");
    if config_path.exists() {
        println!("already exists at {}", config_path.display());
    } else {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let example = include_str!("../config/config.example.toml");
        match std::fs::write(&config_path, example) {
            Ok(()) => println!("created at {}", config_path.display()),
            Err(e) => println!("FAILED ({})", e),
        }
    }

    // 2. Detect shell and install hooks
    let shell = std::env::var("SHELL").unwrap_or_default();
    let (rc_file, hook_file) = if shell.ends_with("zsh") {
        (".zshrc", "hooks.zsh")
    } else {
        (".bashrc", "hooks.bash")
    };

    let home = dirs::home_dir().expect("cannot determine home directory");
    let rc_path = home.join(rc_file);
    let data_dir = home.join(".local/share/npushell");
    let source_line = format!("source {}/{}", data_dir.display(), hook_file);

    print!("Shell hooks (~/{})... ", rc_file);
    if rc_path.exists() {
        let contents = std::fs::read_to_string(&rc_path).unwrap_or_default();
        if contents.contains("npushell") {
            println!("already installed");
        } else {
            print!("not found. Add hook to ~/{}? [Y/n] ", rc_file);
            io::stdout().flush().ok();
            let mut answer = String::new();
            io::stdin().lock().read_line(&mut answer).ok();
            if answer.trim().is_empty() || answer.trim().to_lowercase().starts_with('y') {
                let addition = format!("\n# npushell - NPU-powered shell copilot\n{}\n", source_line);
                match std::fs::OpenOptions::new().append(true).open(&rc_path) {
                    Ok(mut f) => {
                        let _ = f.write_all(addition.as_bytes());
                        println!("  Added to ~/{}", rc_file);
                    }
                    Err(e) => println!("  FAILED ({})", e),
                }
            } else {
                println!("  Skipped. Add manually: {}", source_line);
            }
        }
    } else {
        println!("~/{} not found, skipping", rc_file);
    }

    // 3. Run doctor
    println!();
    handle_doctor(config, client);

    println!("\nSetup complete! Restart your shell or run: source ~/{}", rc_file);
}

fn handle_doctor(config: &Config, client: &CopilotClient) {
    println!("npushell doctor");
    println!("===============\n");

    // Check API connectivity
    print!("API endpoint ({})... ", config.api.endpoint);
    match client.health_check() {
        Ok(()) => println!("OK"),
        Err(e) => println!("FAIL ({})", e),
    }

    // Check config file
    let config_path = Config::config_path();
    print!("Config file ({})... ", config_path.display());
    if config_path.exists() {
        println!("found");
    } else {
        println!("not found (using defaults)");
    }

    // Check shell hook installation
    println!();
    println!("Shell hook status:");
    check_hook_file("bash", ".bashrc");
    check_hook_file("zsh", ".zshrc");

    println!();
    config.display();
}

/// Find a similar string from candidates using edit distance.
/// Returns Some if the best match is within 2 edits.
fn find_similar<'a>(input: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let input_lower = input.to_lowercase();
    let mut best: Option<(&str, usize)> = None;

    for &candidate in candidates {
        if input_lower == candidate {
            return None; // Exact match, not a typo
        }
        let dist = edit_distance(&input_lower, candidate);
        if dist <= 2 {
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((candidate, dist));
            }
        }
    }

    best.map(|(s, _)| s)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];

    for i in 0..=a.len() {
        dp[i][0] = i;
    }
    for j in 0..=b.len() {
        dp[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}

fn check_hook_file(shell_name: &str, rc_file: &str) {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(rc_file);
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if contents.contains("npushell") {
                    println!("  {}: hook installed", shell_name);
                } else {
                    println!("  {}: hook NOT installed in ~/{}", shell_name, rc_file);
                }
            } else {
                println!("  {}: could not read ~/{}", shell_name, rc_file);
            }
        } else {
            println!("  {}: ~/{} not found", shell_name, rc_file);
        }
    }
}
