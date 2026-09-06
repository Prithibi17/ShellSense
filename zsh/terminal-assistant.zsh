# terminal-assistant.zsh - Native Zsh integration for Terminal Assistant
# Zero-lag, rock-solid autocompletion: Tab or Right-Arrow autocompletes.

[[ -o interactive ]] || return 0

# Locate terminal-assistant binary
__terminal_assistant_bin() {
    if (( $+commands[terminal-assistant] )); then
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

typeset -g __terminal_assistant_orig=""
typeset -ga __terminal_assistant_cycle_list=()
typeset -gi __terminal_assistant_cycle_idx=1

# Reset on new prompt
__terminal_assistant_precmd() {
    __terminal_assistant_orig=""
    __terminal_assistant_cycle_list=()
    __terminal_assistant_cycle_idx=1
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd __terminal_assistant_precmd

# Smart Tab autocomplete
__terminal_assistant_tab() {
    local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ -z "$trimmed" ]]; then
        zle expand-or-complete
        return
    fi

    # Cycling
    local total=${#__terminal_assistant_cycle_list[@]}
    if (( total > 1 )) && [[ "$trimmed" == "${__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]}" ]]; then
        __terminal_assistant_cycle_idx=$(( (__terminal_assistant_cycle_idx % total) + 1 ))
        BUFFER="${__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]}"
        CURSOR=${#BUFFER}
        zle redisplay
        return
    fi

    local bin
    bin="$(__terminal_assistant_bin)"
    if [[ -n "$bin" ]]; then
        local raw_output
        raw_output="$("$bin" suggest --cwd "$PWD" --shell zsh --raw-all "$trimmed" 2>/dev/null)"
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

            if (( ${#__terminal_assistant_cycle_list[@]} > 0 )); then
                __terminal_assistant_cycle_idx=1
                local sug="${__terminal_assistant_cycle_list[1]}"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    BUFFER="$sug"
                    CURSOR=${#BUFFER}
                    zle redisplay
                    ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
                    return
                fi
            fi
        fi
    fi

    # Fallback to standard Zsh completion
    zle expand-or-complete
}
zle -N __terminal_assistant_tab

# Smart Right Arrow autocomplete
__terminal_assistant_right() {
    if (( CURSOR >= ${#BUFFER} )); then
        local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
        trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
        if [[ -n "$trimmed" ]]; then
            local bin
            bin="$(__terminal_assistant_bin)"
            if [[ -n "$bin" ]]; then
                local sug
                sug="$("$bin" suggest --cwd "$PWD" --shell zsh --raw "$trimmed" 2>/dev/null)"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    __terminal_assistant_orig="$trimmed"
                    BUFFER="$sug"
                    CURSOR=${#BUFFER}
                    zle redisplay
                    ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
                    return
                fi
            fi
        fi
    fi
    zle forward-char
}
zle -N __terminal_assistant_right

# Escape / Undo
__terminal_assistant_revert() {
    if [[ -n "$__terminal_assistant_orig" ]]; then
        BUFFER="$__terminal_assistant_orig"
        CURSOR=${#BUFFER}
        __terminal_assistant_orig=""
        zle redisplay
    else
        zle send-break
    fi
}
zle -N __terminal_assistant_revert

# Register Zsh keybindings
bindkey '^I' __terminal_assistant_tab
bindkey '^[[C' __terminal_assistant_right
bindkey '^[OA' backward-char 2>/dev/null
bindkey '\e[1;3B' __terminal_assistant_tab
bindkey '\e' __terminal_assistant_revert
