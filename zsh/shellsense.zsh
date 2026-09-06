# shellsense.zsh - Native Zsh integration for ShellSense
# Zero-lag, rock-solid autocompletion: Tab or Right-Arrow autocompletes.

[[ -o interactive ]] || return 0

# Locate shellsense binary
__shellsense_bin() {
    if (( $+commands[shellsense] )); then
        echo "shellsense"
    elif [[ -x "$HOME/.local/bin/shellsense" ]]; then
        echo "$HOME/.local/bin/shellsense"
    elif [[ -x "$HOME/.cargo/bin/shellsense" ]]; then
        echo "$HOME/.cargo/bin/shellsense"
    elif [[ -x "/usr/local/bin/shellsense" ]]; then
        echo "/usr/local/bin/shellsense"
    elif [[ -x "/usr/bin/shellsense" ]]; then
        echo "/usr/bin/shellsense"
    else
        return 1
    fi
}

typeset -g __shellsense_orig=""
typeset -ga __shellsense_cycle_list=()
typeset -gi __shellsense_cycle_idx=1

# Reset on new prompt
__shellsense_precmd() {
    __shellsense_orig=""
    __shellsense_cycle_list=()
    __shellsense_cycle_idx=1
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd __shellsense_precmd

# Smart Tab autocomplete
__shellsense_tab() {
    local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ -z "$trimmed" ]]; then
        zle expand-or-complete
        return
    fi

    # Cycling
    local total=${#__shellsense_cycle_list[@]}
    if (( total > 1 )) && [[ "$trimmed" == "${__shellsense_cycle_list[$__shellsense_cycle_idx]}" ]]; then
        __shellsense_cycle_idx=$(( (__shellsense_cycle_idx % total) + 1 ))
        BUFFER="${__shellsense_cycle_list[$__shellsense_cycle_idx]}"
        CURSOR=${#BUFFER}
        zle redisplay
        return
    fi

    local bin
    bin="$(__shellsense_bin)"
    if [[ -n "$bin" ]]; then
        local raw_output
        raw_output="$("$bin" suggest --cwd "$PWD" --shell zsh --raw-all "$trimmed" 2>/dev/null)"
        if [[ -n "$raw_output" ]]; then
            __shellsense_orig="$trimmed"
            __shellsense_cycle_list=()

            while IFS=$'\t' read -r cat cmd desc risk; do
                if [[ -n "$cmd" ]]; then
                    __shellsense_cycle_list+=("$cmd")
                elif [[ -n "$cat" ]]; then
                    __shellsense_cycle_list+=("$cat")
                fi
            done <<< "$raw_output"

            if (( ${#__shellsense_cycle_list[@]} > 0 )); then
                __shellsense_cycle_idx=1
                local sug="${__shellsense_cycle_list[1]}"
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
zle -N __shellsense_tab

# Smart Right Arrow autocomplete
__shellsense_right() {
    if (( CURSOR >= ${#BUFFER} )); then
        local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
        trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
        if [[ -n "$trimmed" ]]; then
            local bin
            bin="$(__shellsense_bin)"
            if [[ -n "$bin" ]]; then
                local sug
                sug="$("$bin" suggest --cwd "$PWD" --shell zsh --raw "$trimmed" 2>/dev/null)"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    __shellsense_orig="$trimmed"
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
zle -N __shellsense_right

# Escape / Undo
__shellsense_revert() {
    if [[ -n "$__shellsense_orig" ]]; then
        BUFFER="$__shellsense_orig"
        CURSOR=${#BUFFER}
        __shellsense_orig=""
        zle redisplay
    else
        zle send-break
    fi
}
zle -N __shellsense_revert

# Register Zsh keybindings
bindkey '^I' __shellsense_tab
bindkey '^[[C' __shellsense_right
bindkey '^[OA' backward-char 2>/dev/null
bindkey '\e[1;3B' __shellsense_tab
bindkey '\e' __shellsense_revert
