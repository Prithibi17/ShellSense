# terminal-assistant.fish - Native fish integration for Terminal Assistant
# Zero-lag, rock-solid autocompletion: Tab or Right-Arrow autocompletes. No Ctrl+Space required!

status is-interactive; or return

# Locate terminal-assistant binary
function __terminal_assistant_bin
    if type -q terminal-assistant
        echo "terminal-assistant"
    else if test -x "$HOME/.local/bin/terminal-assistant"
        echo "$HOME/.local/bin/terminal-assistant"
    else if test -x "./target/release/terminal-assistant"
        echo "./target/release/terminal-assistant"
    else if test -x "$HOME/.cargo/bin/terminal-assistant"
        echo "$HOME/.cargo/bin/terminal-assistant"
    else if test -x "/usr/local/bin/terminal-assistant"
        echo "/usr/local/bin/terminal-assistant"
    else if test -x "/usr/bin/terminal-assistant"
        echo "/usr/bin/terminal-assistant"
    else
        return 1
    end
end

# Global state for cycling and undo
set -g __terminal_assistant_orig ""
set -g __terminal_assistant_cycle_list
set -g __terminal_assistant_cycle_idx 1

# Reset state on new prompt
function __terminal_assistant_reset --on-event fish_prompt
    set -g __terminal_assistant_orig ""
    set -g __terminal_assistant_cycle_list
    set -g __terminal_assistant_cycle_idx 1
end

# Smart Tab: Autocompletes natural language / AI suggestions, cycles on repeated Tab, or falls back to fish complete
function __terminal_assistant_tab
    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")

    # If buffer is empty, use standard fish tab completion
    if test -z "$trimmed"
        commandline -f complete
        return
    end

    # If already cycling candidates for the current intent
    set -l total (count $__terminal_assistant_cycle_list)
    if test "$total" -gt 0 -a "$trimmed" = "$__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]"
        set -g __terminal_assistant_cycle_idx (math "($__terminal_assistant_cycle_idx % $total) + 1")
        commandline -r "$__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
        return
    end

    # Query terminal-assistant daemon
    set -l bin (__terminal_assistant_bin)
    if test -n "$bin"
        set -l lines ($bin suggest --cwd "$PWD" --shell fish --raw-all "$trimmed" 2>/dev/null)
        if test (count $lines) -gt 0
            set -g __terminal_assistant_orig "$trimmed"
            set -g __terminal_assistant_cycle_list
            set -l is_destructive 0

            for line in $lines
                set -l parts (string split \t -- "$line")
                if test (count $parts) -ge 2
                    set -a __terminal_assistant_cycle_list $parts[2]
                    if test (count $parts) -ge 4 -a "$parts[4]" = "dest"
                        set is_destructive 1
                    end
                else
                    set -a __terminal_assistant_cycle_list "$line"
                end
            end

            set -g __terminal_assistant_cycle_idx 1
            set -l sug $__terminal_assistant_cycle_list[1]

            if test -n "$sug" -a "$sug" != "$trimmed"
                if test "$is_destructive" = "1"
                    echo ""
                    echo -e "\e[1;38;5;203m⚠ Notice: Destructive command suggested. Review carefully before pressing Enter.\e[0m"
                end

                commandline -r "$sug"
                commandline -f end-of-line
                commandline -f repaint

                # Asynchronously record accepted command for local learning
                $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
                return
            end
        end
    end

    # Fallback to standard fish tab completion
    commandline -f complete
end

# Smart Right Arrow: Accepts suggestion when cursor is at the end of the line
function __terminal_assistant_right_arrow
    set -l current_cmd (commandline)
    set -l cursor_pos (commandline -C)
    set -l line_len (string length "$current_cmd")

    # If cursor is at the end of the line, check for smart autocomplete
    if test "$cursor_pos" -ge "$line_len"
        set -l trimmed (string trim "$current_cmd")

        # Skip if already displaying a cycled candidate
        set -l total (count $__terminal_assistant_cycle_list)
        if test "$total" -gt 0 -a "$trimmed" = "$__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]"
            commandline -f forward-char
            return
        end

        set -l bin (__terminal_assistant_bin)
        if test -n "$bin" -a -n "$trimmed"
            set -l sug ($bin suggest --cwd "$PWD" --shell fish --raw "$trimmed" 2>/dev/null)
            if test -n "$sug" -a "$sug" != "$trimmed"
                set -g __terminal_assistant_orig "$trimmed"
                set -g __terminal_assistant_cycle_list "$sug"
                set -g __terminal_assistant_cycle_idx 1

                commandline -r "$sug"
                commandline -f end-of-line
                commandline -f repaint

                $bin record --input "$trimmed" --command "$sug" >/dev/null 2>&1 &
                return
            end
        end
    end

    # Fallback to standard fish forward-char (preserves fish autosuggestion)
    commandline -f forward-char
end

# Escape key: Reverts back to original query if autocompleted, otherwise standard cancel
function __terminal_assistant_revert
    if test -n "$__terminal_assistant_orig"
        commandline -r "$__terminal_assistant_orig"
        set -g __terminal_assistant_orig ""
        set -g __terminal_assistant_cycle_list
        set -g __terminal_assistant_cycle_idx 1
        commandline -f end-of-line
        commandline -f repaint
    else
        commandline -f cancel
    end
end

# Cycle candidates: Alt+Down / Alt+Up
function __terminal_assistant_cycle_next
    set -l total (count $__terminal_assistant_cycle_list)
    if test "$total" -gt 1
        set -g __terminal_assistant_cycle_idx (math "($__terminal_assistant_cycle_idx % $total) + 1")
        commandline -r "$__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
    end
end

function __terminal_assistant_cycle_prev
    set -l total (count $__terminal_assistant_cycle_list)
    if test "$total" -gt 1
        set -g __terminal_assistant_cycle_idx (math "(($__terminal_assistant_cycle_idx - 2 + $total) % $total) + 1")
        commandline -r "$__terminal_assistant_cycle_list[$__terminal_assistant_cycle_idx]"
        commandline -f end-of-line
        commandline -f repaint
    end
end


# Register keybindings
function __terminal_assistant_bind_keys
    # Tab: Smart autocomplete or standard fish complete
    bind \t __terminal_assistant_tab
    bind -M insert \t __terminal_assistant_tab 2>/dev/null

    # Right arrow: Smart autocomplete at end of line, or forward-char
    bind '\e[C' __terminal_assistant_right_arrow 2>/dev/null
    bind -M insert '\e[C' __terminal_assistant_right_arrow 2>/dev/null
    bind right __terminal_assistant_right_arrow 2>/dev/null
    bind -M insert right __terminal_assistant_right_arrow 2>/dev/null

    # Escape: Undo/revert to original query or cancel
    bind \e __terminal_assistant_revert
    bind -M insert \e __terminal_assistant_revert 2>/dev/null

    # Cycling: Alt+Down / Alt+Up
    bind alt-down __terminal_assistant_cycle_next 2>/dev/null
    bind -M insert alt-down __terminal_assistant_cycle_next 2>/dev/null
    bind alt-up __terminal_assistant_cycle_prev 2>/dev/null
    bind -M insert alt-up __terminal_assistant_cycle_prev 2>/dev/null
    bind \e'[1;3B' __terminal_assistant_cycle_next 2>/dev/null
    bind -M insert \e'[1;3B' __terminal_assistant_cycle_next 2>/dev/null
    bind \e'[1;3A' __terminal_assistant_cycle_prev 2>/dev/null
    bind -M insert \e'[1;3A' __terminal_assistant_cycle_prev 2>/dev/null
end

__terminal_assistant_bind_keys
