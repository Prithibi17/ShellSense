# shellsense.bash - Native Bash integration for ShellSense
# Real-time detection before Tab: as you type words, ShellSense displays
# ghost suggestions (→ command) inline. Hit Tab/Right-Arrow to accept or Enter to run.

[[ $- == *i* ]] || return 0

# Locate shellsense binary
__shellsense_bin() {
    if command -v shellsense >/dev/null 2>&1; then
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

__shellsense_hint=""
__shellsense_last_query=""
__shellsense_orig=""
__shellsense_cycle_list=()
__shellsense_cycle_idx=0

# Reset cycle on new prompt
__shellsense_prompt_command() {
    __shellsense_hint=""
    __shellsense_last_query=""
    __shellsense_orig=""
    __shellsense_cycle_list=()
    __shellsense_cycle_idx=0
}
if [[ "$PROMPT_COMMAND" != *"__shellsense_prompt_command"* ]]; then
    PROMPT_COMMAND="__shellsense_prompt_command${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
fi

# Detect intent when space is pressed
__shellsense_space() {
    READLINE_LINE="${READLINE_LINE:0:$READLINE_POINT} ${READLINE_LINE:$READLINE_POINT}"
    READLINE_POINT=$((READLINE_POINT + 1))

    local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ ${#trimmed} -ge 2 ]]; then
        local bin="$(__shellsense_bin)"
        if [[ -n "$bin" ]]; then
            local sug
            sug="$("$bin" suggest --cwd "$PWD" --shell bash --raw "$trimmed" 2>/dev/null)"
            if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                __shellsense_hint="$sug"
                printf "\0337\033[38;5;244m  → %s\033[0m\033[K\0338" "$sug"
                return
            fi
        fi
    fi

    printf "\0337\033[K\0338"
    __shellsense_hint=""
}

# Smart Tab autocomplete
__shellsense_tab() {
    printf "\0337\033[K\0338"
    if [[ -n "$__shellsense_hint" ]]; then
        READLINE_LINE="$__shellsense_hint"
        READLINE_POINT=${#READLINE_LINE}
        local bin="$(__shellsense_bin)"
        local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
        trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
        [[ -n "$bin" ]] && ("$bin" record --input "$trimmed" --command "$__shellsense_hint" >/dev/null 2>&1 &)
        __shellsense_hint=""
        return
    fi

    local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ -z "$trimmed" ]]; then
        return
    fi

    # Check cycling if already active
    local total=${#__shellsense_cycle_list[@]}
    if [[ $total -gt 1 && "$trimmed" == "${__shellsense_cycle_list[$__shellsense_cycle_idx]}" ]]; then
        __shellsense_cycle_idx=$(( (__shellsense_cycle_idx + 1) % total ))
        READLINE_LINE="${__shellsense_cycle_list[$__shellsense_cycle_idx]}"
        READLINE_POINT=${#READLINE_LINE}
        return
    fi

    local bin
    bin="$(__shellsense_bin)"
    if [[ -n "$bin" ]]; then
        local raw_output
        raw_output="$("$bin" suggest --cwd "$PWD" --shell bash --raw-all "$trimmed" 2>/dev/null)"
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

            if [[ ${#__shellsense_cycle_list[@]} -gt 0 ]]; then
                __shellsense_cycle_idx=0
                local sug="${__shellsense_cycle_list[0]}"
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
__shellsense_right() {
    if [[ $READLINE_POINT -ge ${#READLINE_LINE} ]]; then
        if [[ -n "$__shellsense_hint" ]]; then
            printf "\0337\033[K\0338"
            READLINE_LINE="$__shellsense_hint"
            READLINE_POINT=${#READLINE_LINE}
            __shellsense_hint=""
            return
        fi

        local trimmed="${READLINE_LINE#"${READLINE_LINE%%[![:space:]]*}"}"
        trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
        if [[ -n "$trimmed" ]]; then
            local bin
            bin="$(__shellsense_bin)"
            if [[ -n "$bin" ]]; then
                local sug
                sug="$("$bin" suggest --cwd "$PWD" --shell bash --raw "$trimmed" 2>/dev/null)"
                if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
                    __shellsense_orig="$trimmed"
                    READLINE_LINE="$sug"
                    READLINE_POINT=${#READLINE_LINE}
                    ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
                    return
                fi
            fi
        fi
    fi

    if [[ $READLINE_POINT -lt ${#READLINE_LINE} ]]; then
        READLINE_POINT=$((READLINE_POINT + 1))
    fi
}

# Undo / Revert suggestion
__shellsense_revert() {
    printf "\0337\033[K\0338"
    __shellsense_hint=""
    if [[ -n "$__shellsense_orig" ]]; then
        READLINE_LINE="$__shellsense_orig"
        READLINE_POINT=${#READLINE_LINE}
        __shellsense_orig=""
    fi
}

# Register Bash readline keybindings
bind -x '" ": __shellsense_space'
bind -x '"\t": __shellsense_tab'
bind -x '"\e[C": __shellsense_right'
bind -x '"\e[1;3B": __shellsense_tab'
bind -x '"\C-g": __shellsense_revert'
