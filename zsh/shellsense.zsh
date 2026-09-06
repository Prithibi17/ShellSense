# shellsense.zsh - Native Zsh integration for ShellSense
# Real-time inline detection before Tab: as you type, ShellSense displays
# ghost suggestions (→ command) inline via POSTDISPLAY. Hit Tab/Right-Arrow to accept or Enter to run.

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
    end
}

typeset -g __shellsense_hint=""
typeset -g __shellsense_last_query=""
typeset -g __shellsense_orig=""
typeset -ga __shellsense_cycle_list=()
typeset -gi __shellsense_cycle_idx=1
typeset -gi __shellsense_is_dest=0

# Reset on new prompt
__shellsense_precmd() {
    POSTDISPLAY=""
    __shellsense_hint=""
    __shellsense_last_query=""
    __shellsense_orig=""
    __shellsense_cycle_list=()
    __shellsense_cycle_idx=1
    __shellsense_is_dest=0
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd __shellsense_precmd

# Real-time detection hook
__shellsense_detect() {
    local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if [[ ${#trimmed} -lt 2 ]]; then
        POSTDISPLAY=""
        __shellsense_hint=""
        __shellsense_last_query=""
        return
    fi

    if [[ "$trimmed" == "$__shellsense_last_query" ]]; then
        return
    fi
    __shellsense_last_query="$trimmed"

    local bin
    bin="$(__shellsense_bin)"
    [[ -n "$bin" ]] || return

    local raw_line
    raw_line="$("$bin" suggest --cwd "$PWD" --shell zsh --raw-all "$trimmed" 2>/dev/null | head -n 1)"
    if [[ -n "$raw_line" ]]; then
        local parts=("${(@s:\t:)raw_line}")
        local sug=""
        local is_dest=0

        if [[ ${#parts[@]} -ge 2 ]]; then
            sug="${parts[2]}"
            if [[ ${#parts[@]} -ge 4 && "${parts[4]}" == "dest" ]]; then
                is_dest=1
            fi
        else
            sug="$raw_line"
        fi

        if [[ -n "$sug" && "$sug" != "$trimmed" ]]; then
            __shellsense_hint="$sug"
            __shellsense_is_dest=$is_dest
            if [[ $is_dest -eq 1 ]]; then
                POSTDISPLAY=$'  \e[38;5;203m⚠ → '"$sug"$'\e[0m'
            else
                POSTDISPLAY=$'  \e[38;5;244m→ '"$sug"$'\e[0m'
            fi
            return
        fi
    fi

    POSTDISPLAY=""
    __shellsense_hint=""
}

__shellsense_self_insert() {
    zle .self-insert
    __shellsense_detect
}
zle -N self-insert __shellsense_self_insert

__shellsense_backward_delete_char() {
    zle .backward-delete-char
    __shellsense_detect
}
zle -N backward-delete-char __shellsense_backward_delete_char

# Smart Tab autocomplete
__shellsense_tab() {
    if [[ -n "$__shellsense_hint" ]]; then
        local sug="$__shellsense_hint"
        POSTDISPLAY=""
        __shellsense_hint=""
        BUFFER="$sug"
        CURSOR=${#BUFFER}
        local bin="$(__shellsense_bin)"
        [[ -n "$bin" ]] && ("$bin" record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &)
        zle redisplay
        return
    fi

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

    local bin="$(__shellsense_bin)"
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

    zle expand-or-complete
}
zle -N __shellsense_tab

# Smart Right Arrow autocomplete
__shellsense_right() {
    if (( CURSOR >= ${#BUFFER} )); then
        if [[ -n "$__shellsense_hint" ]]; then
            local sug="$__shellsense_hint"
            POSTDISPLAY=""
            __shellsense_hint=""
            BUFFER="$sug"
            CURSOR=${#BUFFER}
            zle redisplay
            return
        fi
    fi
    zle forward-char
}
zle -N __shellsense_right

# Direct Enter execution
__shellsense_accept_line() {
    local trimmed="${BUFFER#"${BUFFER%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
    local first_token="${trimmed%% *}"

    if [[ -n "$__shellsense_hint" && -n "$first_token" ]]; then
        if ! command -v "$first_token" >/dev/null 2>&1; then
            if (( __shellsense_is_dest == 1 )); then
                BUFFER="$__shellsense_hint"
                POSTDISPLAY=""
                __shellsense_hint=""
                CURSOR=${#BUFFER}
                zle redisplay
                echo ""
                print -P "%F{red}⚠ Notice: Destructive command detected. Press Enter to execute, or Ctrl+C to cancel.%f"
                return
            else
                BUFFER="$__shellsense_hint"
                local bin="$(__shellsense_bin)"
                [[ -n "$bin" ]] && ("$bin" record --input "$trimmed" --command "$__shellsense_hint" >/dev/null 2>&1 &)
            fi
        fi
    fi

    POSTDISPLAY=""
    __shellsense_hint=""
    zle .accept-line
}
zle -N accept-line __shellsense_accept_line

# Escape / Undo
__shellsense_revert() {
    POSTDISPLAY=""
    __shellsense_hint=""
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
bindkey '\e[1;3B' __shellsense_tab
bindkey '\e' __shellsense_revert
