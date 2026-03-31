# npushell

An NPU-powered shell copilot that runs entirely on your local machine. It uses your NPU (or any local LLM server) to automatically fix failed commands, explain commands, and answer shell questions — with zero cloud dependency.

## How it works

1. Shell hooks silently monitor your commands
2. When a command fails, `npushell` calls your local LLM in the **background**
3. At your next prompt, the suggested fix appears — you choose whether to run it
4. You can also query it directly: `npu "how to find large files"`

Your workflow is **never blocked** — suggestions arrive asynchronously.

## Features

- **Auto-fix** — failed commands get diagnosed and a corrected command is suggested at the next prompt
- **Explain** — break down what a command does, flag by flag
- **Suggest** — describe what you want in plain English, get the command
- **Ask** — free-form shell/Linux questions
- **Works with any OpenAI-compatible API** — lemonade-server, ollama, llama.cpp, vLLM, etc.
- **Bash & Zsh** support
- **Non-blocking** — LLM calls run in the background

## Requirements

- Rust toolchain (for building)
- A local LLM server with an OpenAI-compatible API (e.g., [lemonade-server](https://github.com/onnx/turnkeyml))
- Bash or Zsh

## Installation

```bash
git clone https://github.com/YOUR_USER/npushell
cd npushell
make install
```

Then add to your shell config:

```bash
# For bash (~/.bashrc):
source ~/.local/share/npushell/hooks.bash

# For zsh (~/.zshrc):
source ~/.local/share/npushell/hooks.zsh
```

Restart your shell and run `npushell doctor` to verify.

## Usage

### Automatic fix (just use your shell normally)

```
$ git pussh origin main
fatal: 'pussh' is not a git command...

$ ↵  (just press Enter)

 npushell ─ suggested fix:
  $ git push origin main
  Typo: 'pussh' should be 'push'.

  Run this command? [Y/n]
```

### Direct queries

```bash
# Explain a command
npu explain "tar -xzf archive.tar.gz -C /tmp"

# Suggest a command
npu suggest "find files larger than 100MB"

# Ask anything
npu "how to list all USB devices"

# Check setup
npushell doctor
```

## Configuration

Create `~/.config/npushell/config.toml`:

```toml
[api]
endpoint = "http://localhost:8000"
model = "default"
timeout = 30

[behavior]
auto_fix = true
confirm_before_run = true
max_history_context = 5

[ui]
color = true
```

See `config/config.example.toml` for all options.

## How it works (technical)

- **Shell hooks** (`preexec`/`precmd`) capture failed commands with near-zero overhead (~6ms bash, ~1ms zsh)
- On failure, `npushell fix` is spawned in the **background** — your prompt returns immediately
- The fix is written to a temp file; the next `precmd` hook picks it up and displays it
- Direct commands (`explain`, `suggest`, `ask`) run in the foreground
- All LLM calls go to your local OpenAI-compatible endpoint — nothing leaves your machine

## License

MIT
