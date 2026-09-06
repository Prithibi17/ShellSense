# terminal-assistant.bash - Native Bash integration for Terminal Assistant
# Zero-lag, rock-solid autocompletion: Tab or Right-Arrow autocompletes.

# Only run in interactive shells
[[ $- == *i* ]] || return 0

# Locate terminal-assistant binary
__terminal_assistant_bin() {
    if command -v terminal-assistant >/dev/null 2>&1; then
        echo "terminal-assistant"
    elif [[ -x "$HOME/.local/bin/terminal-assistant" ]]; then
        echo "$HOME/.local/bin/terminal-assistant"
    elif [[ -x "$HOME/.cargo/bin/terminal-assistant" ]]; then
        echo "$HOME/.cargo/bin/terminal-assistant"
    elif [[ -x "/usr/local/bin/terminal-assistant" ]]; then
        echo "/usr/local/bin/terminal-assistant"
    elif [[ -x "/usr/bin/terminal-assistant" ]]; then
        echo "/usr/bin/terminal-assistant"
    else
        return 1
    fi
}

__terminal_assistant_orig=""
__terminal_assistant_cycle_list=()
__terminal_assistant_cycle_idx=0

# Reset cycle on new prompt
__terminal_assistant_prompt_command() {
    __terminal_assistant_orig=""
    __terminal_assistant_cycle_list=()
    __terminal_assistant_cycle_idx=0
}
if [[ "$PROMPT_COMMAND" != *"__terminal_assistant_prompt_command"* ]]; then
    PROMPT_COMMAND="__terminal_assistant_prompt_command${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
fi

# Smart Tab autocomplete
__terminal_assistant_tab() {
    local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ -z "$trimmed" ]]; then
        return
    fi

    # Check cycling if already active
    local total=${#__terminal_assistant_cycle_list[@]}
    if [[ $total -gt 1 && "$trimmed" == "${__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]}" ]]; then
        __terminal_assistant_cycle_idx=$(( (__terminal_assistant_cycle_idx + 1) % total ))
        READLINE_LINE="${__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]}"
        READLINE_POINT=${#READLINE_LINE}
        return
    fi

    local bin
    bin="$(__terminal_assistant_bin)"
    if [[ -n "$bin" ]]; then
        local raw_output
        raw_output="$("$bin" suggest --cwd "$PWD" --shell bash --raw-all "$trimmed" 2>/dev/null)"
        if [[ -n "$raw_output" ]]; then
            __terminal_assistant_orig="$trimmed"
            __terminal_assistant_cycle_list=()

            while IFS=$'\t' read -r cat cmd desc risk; do
                if [[ -n "$cmd" ]]; then
                    __terminal_assistant_cycle_list+=("$cmd")
                elif [[ -n "$cat" ]]; then
                    __terminal_assistant_cycle_list+=("$cat")
                fi
            done <<< "$raw_output"

            if [[ ${#__terminal_assistant_cycle_list[@]} -gt 0 ]]; then
                __terminal_assistant_cycle_idx=0
                local sug="${__terminal_assistant_cycle_list[0]}"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    READLINE_LINE="$sug"
                    READLINE_POINT=${#READLINE_LINE}
                    ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
                    return
                fi
            fi
        fi
    fi

    # Fallback to standard Bash path/command completion
    local cur="${READLINE_LINE:0:READLINE_POINT}"
    cur="${cur##* }"
    if [[ -n "$cur" ]]; then
        local matches=($(compgen -f -- "$cur" 2>/dev/null))
        if [[ ${#matches[@]} -eq 1 ]]; then
            local match="${matches[0]}"
            if [[ -d "$match" ]]; then
                match="$match/"
            fi
            READLINE_LINE="${READLINE_LINE:0:$((READLINE_POINT - ${#cur}))}$match"
            READLINE_POINT=${#READLINE_LINE}
        elif [[ ${#matches[@]} -gt 1 ]]; then
            printf "\n"
            printf "%s  " "${matches[@]}"
            printf "\n"
        fi
    fi
}

# Smart Right Arrow autocomplete
__terminal_assistant_right() {
    if [[ $READLINE_POINT -ge ${#READLINE_LINE} ]]; then
        local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
        trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
        if [[ -n "$trimmed" ]]; then
            local bin
            bin="$(__terminal_assistant_bin)"
            if [[ -n "$bin" ]]; then
                local sug
                sug="$("$bin" suggest --cwd "$PWD" --shell bash --raw "$trimmed" 2>/dev/null)"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    __terminal_assistant_orig="$trimmed"
                    READLINE_LINE="$sug"
                    READLINE_POINT=${#READLINE_LINE}
                    ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
                    return
                fi
            fi
        fi
    fi
    # If not completing, advance cursor forward by 1 character
    if [[ $READLINE_POINT -lt ${#READLINE_LINE} ]]; then
        READLINE_POINT=$((READLINE_POINT + 1))
    fi
}

# Undo / Revert suggestion
__terminal_assistant_revert() {
    if [[ -n "$__terminal_assistant_orig" ]]; then
        READLINE_LINE="$__terminal_assistant_orig"
        READLINE_POINT=${#READLINE_LINE}
        __terminal_assistant_orig=""
    fi
}

# Register Bash readline keybindings
bind -x '"\t": __terminal_assistant_tab'
bind -x '"\e[C": __terminal_assistant_right'
bind -x '"\e[1;3B": __terminal_assistant_tab'
bind -x '"\C-g": __terminal_assistant_revert'
