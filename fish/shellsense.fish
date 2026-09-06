# shellsense.fish - Native fish integration for ShellSense
# Real-time inline detection before Tab: as you type, ShellSense displays
# ghost suggestions (→ command) inline. Hit Tab/Right-Arrow to accept or Enter to run.

status is-interactive; or return

# Locate shellsense binary
function __shellsense_bin
    if type -q shellsense
        echo "shellsense"
    else if test -x "$HOME/.local/bin/shellsense"
        echo "$HOME/.local/bin/shellsense"
    else if test -x "$HOME/.cargo/bin/shellsense"
        echo "$HOME/.cargo/bin/shellsense"
    else if test -x "./target/release/shellsense"
        echo "./target/release/shellsense"
    else if test -x "/usr/local/bin/shellsense"
        echo "/usr/local/bin/shellsense"
    else if test -x "/usr/bin/shellsense"
        echo "/usr/bin/shellsense"
    else
        return 1
    end
end

# Global state for inline hints and candidate cycling
set -g __shellsense_hint ""
set -g __shellsense_last_query ""
set -g __shellsense_orig ""
set -g __shellsense_cycle_list
set -g __shellsense_cycle_idx 1
set -g __shellsense_is_destructive 0

# Helper: clear inline ghost hint
function __shellsense_clear_hint
    if test -n "$__shellsense_hint"
        # Save cursor (VT100 \e7), clear to end of line (\e[K), restore cursor (\e8)
        printf "\0337\033[K\0338"
        set -g __shellsense_hint ""
        set -g __shellsense_is_destructive 0
    end
end

# Reset state on each new prompt
function __shellsense_reset --on-event fish_prompt
    __shellsense_clear_hint
    set -g __shellsense_hint ""
    set -g __shellsense_last_query ""
    set -g __shellsense_orig ""
    set -g __shellsense_cycle_list
    set -g __shellsense_cycle_idx 1
    set -g __shellsense_is_destructive 0
end

# Real-time detection: invoked on every keystroke
function __shellsense_detect
    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")

    # If buffer is too short, clear ghost hint
    if test (string length "$trimmed") -lt 2
        __shellsense_clear_hint
        set -g __shellsense_last_query ""
        return
    end

    # Skip query if unchanged
    if test "$trimmed" = "$__shellsense_last_query"
        return
    end
    set -g __shellsense_last_query "$trimmed"

    # Fast sub-millisecond query to daemon
    set -l bin (__shellsense_bin)
    or return

    set -l raw_line ($bin suggest --cwd "$PWD" --shell fish --raw-all "$trimmed" 2>/dev/null | head -n 1)
    if test -n "$raw_line"
        set -l parts (string split \t -- "$raw_line")
        set -l sug ""
        set -l is_dest 0

        if test (count $parts) -ge 2
            set sug $parts[2]
            if test (count $parts) -ge 4 -a "$parts[4]" = "dest"
                set is_dest 1
            end
        else
            set sug "$raw_line"
        end

        if test -n "$sug" -a "$sug" != "$trimmed"
            set -g __shellsense_hint "$sug"
            set -g __shellsense_is_destructive $is_dest

            # Subtle dim gray ghost text preview right after cursor
            if test "$is_dest" = "1"
                printf "\0337\033[38;5;203m  ⚠ → %s\033[0m\033[K\0338" "$sug"
            else
                printf "\0337\033[38;5;244m  → %s\033[0m\033[K\0338" "$sug"
            end
            return
        end
    end

    __shellsense_clear_hint
end

# Smart Tab: accepts detected suggestion, cycles candidates, or falls back to fish complete
function __shellsense_tab
    if test -n "$__shellsense_hint"
        set -l sug "$__shellsense_hint"
        set -l bin (__shellsense_bin)
        set -l trimmed (string trim (commandline))
        __shellsense_clear_hint

        set -g __shellsense_orig "$trimmed"
        commandline -r "$sug"
        commandline -f end-of-line
        commandline -f repaint

        if test -n "$bin" -a -n "$trimmed"
            $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
        end
        return
    end

    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")

    if test -z "$trimmed"
        commandline -f complete
        return
    end

    # If already cycling candidates for the current intent
    set -l total (count $__shellsense_cycle_list)
    if test "$total" -gt 0 -a "$trimmed" = "$__shellsense_cycle_list[$__shellsense_cycle_idx]"
        set -g __shellsense_cycle_idx (math "($__shellsense_cycle_idx % $total) + 1")
        commandline -r "$__shellsense_cycle_list[$__shellsense_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
        return
    end

    # Query full candidate list
    set -l bin (__shellsense_bin)
    if test -n "$bin"
        set -l lines ($bin suggest --cwd "$PWD" --shell fish --raw-all "$trimmed" 2>/dev/null)
        if test (count $lines) -gt 0
            set -g __shellsense_orig "$trimmed"
            set -g __shellsense_cycle_list

            for line in $lines
                set -l parts (string split \t -- "$line")
                if test (count $parts) -ge 2
                    set -a __shellsense_cycle_list $parts[2]
                else
                    set -a __shellsense_cycle_list "$line"
                end
            end

            set -g __shellsense_cycle_idx 1
            set -l sug $__shellsense_cycle_list[1]
            if test -n "$sug" -a "$sug" != "$trimmed"
                commandline -r "$sug"
                commandline -f end-of-line
                commandline -f repaint
                $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
                return
            end
        end
    end

    commandline -f complete
end

# Smart Right Arrow: Accepts suggestion when cursor is at the end of the line
function __shellsense_right_arrow
    set -l current_cmd (commandline)
    set -l cursor_pos (commandline -C)
    set -l line_len (string length "$current_cmd")

    if test "$cursor_pos" -ge "$line_len"
        if test -n "$__shellsense_hint"
            set -l sug "$__shellsense_hint"
            set -l bin (__shellsense_bin)
            set -l trimmed (string trim "$current_cmd")
            __shellsense_clear_hint

            set -g __shellsense_orig "$trimmed"
            commandline -r "$sug"
            commandline -f end-of-line
            commandline -f repaint

            if test -n "$bin" -a -n "$trimmed"
                $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
            end
            return
        end
    end

    commandline -f forward-char
end

# Direct Enter execution: If intent was detected and typed command is NOT an executable, auto-substitute
function __shellsense_enter
    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")
    set -l first_token (string split ' ' -- "$trimmed")[1]

    if test -n "$__shellsense_hint" -a -n "$first_token"
        # Only substitute if the user's typed command doesn't exist as an actual binary/alias
        if not type -q "$first_token"
            if test "$__shellsense_is_destructive" = "1"
                # For destructive commands, do not blindly auto-run; put in buffer for explicit review
                commandline -r "$__shellsense_hint"
                __shellsense_clear_hint
                commandline -f end-of-line
                commandline -f repaint
                echo ""
                echo -e "\033[1;38;5;203m⚠ Notice: Destructive command detected. Press Enter to execute, or Ctrl+C to cancel.\033[0m"
                return
            else
                set -l bin (__shellsense_bin)
                set -l sug "$__shellsense_hint"
                commandline -r "$sug"
                if test -n "$bin"
                    $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
                end
            end
        end
    end

    __shellsense_clear_hint
    commandline -f execute
end

# Escape key: Reverts back to original query or clears
function __shellsense_revert
    __shellsense_clear_hint
    if test -n "$__shellsense_orig"
        commandline -r "$__shellsense_orig"
        set -g __shellsense_orig ""
        set -g __shellsense_cycle_list
        set -g __shellsense_cycle_idx 1
        commandline -f end-of-line
        commandline -f repaint
    else
        commandline -f cancel
    end
end

# Cycle candidates (Alt+Down / Alt+Up)
function __shellsense_cycle_next
    set -l total (count $__shellsense_cycle_list)
    if test "$total" -gt 1
        set -g __shellsense_cycle_idx (math "($__shellsense_cycle_idx % $total) + 1")
        commandline -r "$__shellsense_cycle_list[$__shellsense_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
    end
end

function __shellsense_cycle_prev
    set -l total (count $__shellsense_cycle_list)
    if test "$total" -gt 1
        set -g __shellsense_cycle_idx (math "(($__shellsense_cycle_idx - 2 + $total) % $total) + 1")
        commandline -r "$__shellsense_cycle_list[$__shellsense_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
    end
end

# Register bindings
function __shellsense_bind_keys
    # Real-time keystroke hooks
    bind '' self-insert __shellsense_detect
    bind -M insert '' self-insert __shellsense_detect 2>/dev/null

    bind space self-insert expand-abbr __shellsense_detect
    bind -M insert space self-insert expand-abbr __shellsense_detect 2>/dev/null

    bind backspace backward-delete-char __shellsense_detect
    bind -M insert backspace backward-delete-char __shellsense_detect 2>/dev/null

    bind \x7f backward-delete-char __shellsense_detect
    bind -M insert \x7f backward-delete-char __shellsense_detect 2>/dev/null

    bind delete delete-char __shellsense_detect 2>/dev/null
    bind -M insert delete delete-char __shellsense_detect 2>/dev/null

    # Tab: accept suggestion or fish complete
    bind tab __shellsense_tab
    bind -M insert tab __shellsense_tab 2>/dev/null

    # Right arrow: accept suggestion at end of line
    bind right __shellsense_right_arrow 2>/dev/null
    bind -M insert right __shellsense_right_arrow 2>/dev/null

    # Enter: auto-substitute unknown intent commands
    bind enter __shellsense_enter
    bind -M insert enter __shellsense_enter 2>/dev/null

    # Escape: clear hint or revert
    bind escape __shellsense_revert 2>/dev/null
    bind -M insert escape __shellsense_revert 2>/dev/null

    # Cycling: Alt+Down / Alt+Up
    bind alt-down __shellsense_cycle_next 2>/dev/null
    bind -M insert alt-down __shellsense_cycle_next 2>/dev/null
    bind alt-up __shellsense_cycle_prev 2>/dev/null
    bind -M insert alt-up __shellsense_cycle_prev 2>/dev/null
end

__shellsense_bind_keys
