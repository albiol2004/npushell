#!/bin/bash
# npushell - bash shell hooks
# Source this file in your ~/.bashrc:
#   source ~/.local/share/npushell/hooks.bash

# Guard against double-loading
[[ -n "$__NPUSHELL_LOADED" ]] && return
__NPUSHELL_LOADED=1

# Source bash-preexec if not already loaded
if [[ -z "$__bp_imported" ]]; then
    _npushell_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    source "${_npushell_dir}/bash-preexec.sh"
fi

# State
__npushell_last_command=""

# preexec: capture command text before execution
__npushell_preexec() {
    __npushell_last_command="$1"
}

# precmd: runs after each command, before prompt
__npushell_precmd() {
    local exit_code=$?

    # 1. Check for pending suggestions from a previous fix
    local suggestion_file="/tmp/npushell-suggestion.$$"
    if [[ -f "$suggestion_file" ]]; then
        local cmd="" explanation=""
        while IFS= read -r line; do
            case "$line" in
                COMMAND:*) cmd="${line#COMMAND:}" ;;
                EXPLANATION:*) explanation="${line#EXPLANATION:}" ;;
            esac
        done < "$suggestion_file"
        rm -f "$suggestion_file"

        if [[ -n "$cmd" ]]; then
            echo ""
            echo -e "\033[1;36m npushell\033[0m \033[2m─ suggested fix:\033[0m"
            echo -e "  \033[1;32m\$ ${cmd}\033[0m"
            if [[ -n "$explanation" ]]; then
                echo -e "  \033[2m${explanation}\033[0m"
            fi
            echo ""
            read -p "  Run this command? [Y/n] " -n 1 -r reply
            echo ""
            if [[ "$reply" =~ ^[Yy]?$ ]]; then
                eval "$cmd"
            fi
        fi
    fi

    # 2. If the last command failed, spawn background fix
    if [[ $exit_code -ne 0 && -n "$__npushell_last_command" ]]; then
        # Don't try to fix our own suggestion prompts or very short commands
        if [[ "$__npushell_last_command" != npushell* && ${#__npushell_last_command} -gt 1 ]]; then
            npushell fix \
                --command "$__npushell_last_command" \
                --exit-code "$exit_code" \
                --shell bash \
                --pid $$ &
            disown
        fi
    fi

    __npushell_last_command=""
}

preexec_functions+=(__npushell_preexec)
precmd_functions+=(__npushell_precmd)

# Convenience alias
npu() {
    command npushell "$@"
}
