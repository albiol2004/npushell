#!/bin/zsh
# npushell - zsh shell hooks
# Source this file in your ~/.zshrc:
#   source ~/.local/share/npushell/hooks.zsh

# Guard against double-loading
[[ -n "$__NPUSHELL_LOADED" ]] && return
__NPUSHELL_LOADED=1

typeset -g __npushell_last_command=""

__npushell_preexec() {
    __npushell_last_command="$1"
}

__npushell_precmd() {
    local exit_code=$?

    # 1. Check for pending suggestions
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
            read -k 1 "reply?  Run this command? [Y/n] "
            echo ""
            if [[ "$reply" =~ ^[Yy]?$ ]]; then
                eval "$cmd"
            fi
        fi
    fi

    # 2. If the last command failed, spawn background fix
    if (( exit_code != 0 )) && [[ -n "$__npushell_last_command" ]]; then
        if [[ "$__npushell_last_command" != npushell* && ${#__npushell_last_command} -gt 1 ]]; then
            npushell fix \
                --command "$__npushell_last_command" \
                --exit-code "$exit_code" \
                --shell zsh \
                --pid $$ &!
        fi
    fi

    __npushell_last_command=""
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __npushell_preexec
add-zsh-hook precmd __npushell_precmd

# Convenience alias
npu() {
    command npushell "$@"
}
