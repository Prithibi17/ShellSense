# terminal-assistant.fish - Native fish integration for Terminal Assistant
# Auto-suggests while typing: Accept with Right-Arrow or Tab. No Ctrl+Space required!

status is-interactive; or exit

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

# Global state
set -g __terminal_assistant_last_input ""
set -g __terminal_assistant_current_suggestion ""
set -g __terminal_assistant_suggestions
set -g __terminal_assistant_index 1
set -g __terminal_assistant_has_ghost 0

# Helper: Clear the ghost line below the cursor
function __terminal_assistant_clear_ghost
    if test "$__terminal_assistant_has_ghost" = "1"
        printf "\e7\r\n\e[2K\e8"
        set -g __terminal_assistant_has_ghost 0
    end
end

# Helper: Render rich aesthetic ghost line beneath the command prompt
function __terminal_assistant_render_ghost -a raw_line
    test -n "$raw_line"; or return

    # Parse tab-separated metadata: CATEGORY \t COMMAND \t DESCRIPTION \t IS_DEST
    set -l parts (string split \t -- "$raw_line")
    set -l cat $parts[1]
    set -l cmd $parts[2]
    set -l desc $parts[3]
    set -l is_dest $parts[4]

    # Fallback if no tabs
    if test -z "$cmd"
        set cmd "$raw_line"
        set cat "Linux"
        set is_dest "ok"
    end

    # Calculate indentation under command start
    set -l prompt_str (fish_prompt 2>/dev/null)
    set -l last_line (echo -n "$prompt_str" | tail -n 1)
    set -l indent_len (string length -V -- "$last_line" 2>/dev/null)
    if test -z "$indent_len" -o "$indent_len" -le 0
        set indent_len 2
    end
    set -l spaces (string repeat -n "$indent_len" " ")

    # Category badge formatting with Caelestia color palette
    set -l cat_badge ""
    switch "$cat"
        case AUR
            set cat_badge "\e[38;5;81m[󰣇 AUR]\e[0m"
        case Pacman
            set cat_badge "\e[38;5;114m[󰮯 Pacman]\e[0m"
        case Systemd
            set cat_badge "\e[38;5;141m[󱓞 Systemd]\e[0m"
        case Hardware
            set cat_badge "\e[38;5;215m[󰢮 Hardware]\e[0m"
        case Audio
            set cat_badge "\e[38;5;75m[󰓃 Audio]\e[0m"
        case Git
            set cat_badge "\e[38;5;208m[󰊢 Git]\e[0m"
        case Project
            set cat_badge "\e[38;5;183m[ Project]\e[0m"
        case Network
            set cat_badge "\e[38;5;43m[󰒋 Network]\e[0m"
        case Storage
            set cat_badge "\e[38;5;178m[󰋊 Storage]\e[0m"
        case Memory
            set cat_badge "\e[38;5;147m[󰍛 Memory]\e[0m"
        case Processes
            set cat_badge "\e[38;5;222m[󰒲 Process]\e[0m"
        case Hyprland Desktop
            set cat_badge "\e[38;5;117m[󰖲 Wayland]\e[0m"
        case Btrfs System
            set cat_badge "\e[38;5;158m[󰋊 Btrfs]\e[0m"
        case '*'
            set cat_badge "\e[38;5;246m[$cat]\e[0m"
    end

    # Multi-candidate suggestion counter
    set -l counter_badge ""
    set -l total (count $__terminal_assistant_suggestions)
    if test "$total" -gt 1
        set counter_badge " \e[38;5;242m[$__terminal_assistant_index/$total · Alt+↓]\e[0m"
    end

    # Key hint
    set -l key_hint " \e[38;5;243m[⇥ Tab]\e[0m"

    # Render line below cursor using ANSI save/restore
    if test "$is_dest" = "dest"
        printf "\e7\r\n\e[2K%s\e[38;5;240m└─ \e[1;38;5;203m⚠ %s\e[0m  \e[1;38;5;203m[⚠ Destructive]\e[0m%b%b\e8" \
            "$spaces" "$cmd" "$counter_badge" "$key_hint"
    else
        printf "\e7\r\n\e[2K%s\e[38;5;240m└─ \e[1;38;5;254m%s\e[0m  %b%b%b\e8" \
            "$spaces" "$cmd" "$cat_badge" "$counter_badge" "$key_hint"
    end

    set -g __terminal_assistant_has_ghost 1
end

# Automatic trigger as user types
function __terminal_assistant_on_change
    set -l bin (__terminal_assistant_bin)
    or return

    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")

    # If input is too short (< 2 chars) or empty, clear suggestion
    if test (string length "$trimmed") -lt 2
        __terminal_assistant_clear_ghost
        set -g __terminal_assistant_current_suggestion ""
        set -g __terminal_assistant_suggestions
        set -g __terminal_assistant_last_input ""
        return
    end

    # If input hasn't changed, keep existing display
    if test "$trimmed" = "$__terminal_assistant_last_input"
        return
    end
    set -g __terminal_assistant_last_input "$trimmed"

    # Query candidate suggestions
    set -l lines ($bin suggest --cwd "$PWD" --shell fish --raw-all "$trimmed" 2>/dev/null)

    if test (count $lines) -gt 0
        set -g __terminal_assistant_suggestions $lines
        set -g __terminal_assistant_index 1
        set -g __terminal_assistant_current_suggestion $lines[1]
        __terminal_assistant_render_ghost "$lines[1]"
    else
        __terminal_assistant_clear_ghost
        set -g __terminal_assistant_current_suggestion ""
        set -g __terminal_assistant_suggestions
    end
end

# Cycle to next suggestion (Alt+Down)
function __terminal_assistant_cycle_next
    set -l count (count $__terminal_assistant_suggestions)
    if test "$count" -gt 1
        set -g __terminal_assistant_index (math "($__terminal_assistant_index % $count) + 1")
        set -g __terminal_assistant_current_suggestion $__terminal_assistant_suggestions[$__terminal_assistant_index]
        __terminal_assistant_render_ghost "$__terminal_assistant_current_suggestion"
    end
end

# Cycle to previous suggestion (Alt+Up)
function __terminal_assistant_cycle_prev
    set -l count (count $__terminal_assistant_suggestions)
    if test "$count" -gt 1
        set -g __terminal_assistant_index (math "(($__terminal_assistant_index - 2 + $count) % $count) + 1")
        set -g __terminal_assistant_current_suggestion $__terminal_assistant_suggestions[$__terminal_assistant_index]
        __terminal_assistant_render_ghost "$__terminal_assistant_current_suggestion"
    end
end

# Accept ghost suggestion (Right arrow at end of line)
function __terminal_assistant_accept_ghost
    if test -n "$__terminal_assistant_current_suggestion"
        set -l current_cmd (commandline)
        set -l cursor_pos (commandline -C)
        set -l line_len (string length "$current_cmd")

        if test "$cursor_pos" -ge "$line_len"
            set -l bin (__terminal_assistant_bin)
            set -l orig "$current_cmd"
            set -l parts (string split \t -- "$__terminal_assistant_current_suggestion")
            set -l suggestion $parts[2]
            if test -z "$suggestion"
                set suggestion "$__terminal_assistant_current_suggestion"
            end

            __terminal_assistant_clear_ghost
            set -g __terminal_assistant_current_suggestion ""
            set -g __terminal_assistant_suggestions

            # Replace command line buffer
            commandline -r "$suggestion"
            commandline -f end-of-line
            commandline -f repaint

            # Record accepted command for learning
            if test -n "$bin"
                $bin record --input "$orig" --command "$suggestion" >/dev/null 2>&1 &
            end
            return
        end
    end

    # Fallback to standard fish forward-char
    commandline -f forward-char
end

# Accept ghost suggestion (Tab key)
function __terminal_assistant_tab_accept
    if test -n "$__terminal_assistant_current_suggestion"
        set -l current_cmd (commandline)
        set -l trimmed (string trim "$current_cmd")
        set -l parts (string split \t -- "$__terminal_assistant_current_suggestion")
        set -l suggestion $parts[2]
        if test -z "$suggestion"
            set suggestion "$__terminal_assistant_current_suggestion"
        end

        if test -n "$trimmed" -a "$trimmed" != "$suggestion"
            set -l bin (__terminal_assistant_bin)
            __terminal_assistant_clear_ghost
            set -g __terminal_assistant_current_suggestion ""
            set -g __terminal_assistant_suggestions

            commandline -r "$suggestion"
            commandline -f end-of-line
            commandline -f repaint

            if test -n "$bin"
                $bin record --input "$trimmed" --command "$suggestion" >/dev/null 2>&1 &
            end
            return
        end
    end

    # Standard fish tab completion
    commandline -f complete
end

# Dismiss ghost suggestion (Escape key)
function __terminal_assistant_dismiss
    __terminal_assistant_clear_ghost
    set -g __terminal_assistant_current_suggestion ""
    set -g __terminal_assistant_suggestions
    commandline -f cancel
    commandline -f repaint
end

# Enter key handler: Cleans ghost line before executing
function __terminal_assistant_on_enter
    __terminal_assistant_clear_ghost
    set -g __terminal_assistant_current_suggestion ""
    set -g __terminal_assistant_suggestions
    commandline -f execute
end

# Cancel handler (Ctrl+C): Cleans ghost line
function __terminal_assistant_on_cancel
    __terminal_assistant_clear_ghost
    set -g __terminal_assistant_current_suggestion ""
    set -g __terminal_assistant_suggestions
    commandline -f cancel-commandline
end

# Manual trigger (Ctrl+Space still available as direct forced replacement)
function __terminal_assistant_manual_suggest
    set -l bin (__terminal_assistant_bin)
    or return

    set -l current_cmd (commandline)
    set -l trimmed (string trim "$current_cmd")
    test -n "$trimmed"; or return

    set -l suggestion ($bin suggest --cwd "$PWD" --shell fish --raw "$trimmed" 2>/dev/null)
    test -n "$suggestion"; or return

    __terminal_assistant_clear_ghost
    set -g __terminal_assistant_current_suggestion ""

    commandline -r "$suggestion"
    commandline -f end-of-line
    commandline -f repaint

    $bin record --input "$trimmed" --command "$suggestion" >/dev/null 2>&1 &
end

# Pre-prompt cleanup hook
function __terminal_assistant_pre_prompt --on-event fish_prompt
    set -g __terminal_assistant_current_suggestion ""
    set -g __terminal_assistant_suggestions
    set -g __terminal_assistant_last_input ""
    set -g __terminal_assistant_has_ghost 0
end

# Bindings registration
function __terminal_assistant_bind_keys
    # Automatic trigger on all typed characters
    bind '' 'self-insert; __terminal_assistant_on_change'
    bind -M insert '' 'self-insert; __terminal_assistant_on_change' 2>/dev/null

    # Deletions trigger auto-recalculation or clearing
    bind backspace 'backward-delete-char; __terminal_assistant_on_change'
    bind -M insert backspace 'backward-delete-char; __terminal_assistant_on_change' 2>/dev/null
    bind \x7f 'backward-delete-char; __terminal_assistant_on_change'
    bind -M insert \x7f 'backward-delete-char; __terminal_assistant_on_change' 2>/dev/null
    bind delete 'delete-char; __terminal_assistant_on_change' 2>/dev/null
    bind -M insert delete 'delete-char; __terminal_assistant_on_change' 2>/dev/null

    # Acceptance keys
    bind '\e[C' __terminal_assistant_accept_ghost 2>/dev/null
    bind -M insert '\e[C' __terminal_assistant_accept_ghost 2>/dev/null
    bind right __terminal_assistant_accept_ghost 2>/dev/null
    bind -M insert right __terminal_assistant_accept_ghost 2>/dev/null

    # Tab accepts ghost suggestion or normal completes
    bind \t __terminal_assistant_tab_accept
    bind -M insert \t __terminal_assistant_tab_accept 2>/dev/null
    bind tab __terminal_assistant_tab_accept 2>/dev/null
    bind -M insert tab __terminal_assistant_tab_accept 2>/dev/null

    # Cycling keys (Alt+Down and Alt+Up)
    bind '\e[1;3B' __terminal_assistant_cycle_next 2>/dev/null
    bind -M insert '\e[1;3B' __terminal_assistant_cycle_next 2>/dev/null
    bind '\e[1;3A' __terminal_assistant_cycle_prev 2>/dev/null
    bind -M insert '\e[1;3A' __terminal_assistant_cycle_prev 2>/dev/null

    # Escape dismisses suggestion
    bind \e __terminal_assistant_dismiss
    bind -M insert \e __terminal_assistant_dismiss 2>/dev/null
    bind escape __terminal_assistant_dismiss 2>/dev/null
    bind -M insert escape __terminal_assistant_dismiss 2>/dev/null

    # Enter executes command cleanly
    bind \r __terminal_assistant_on_enter
    bind -M insert \r __terminal_assistant_on_enter 2>/dev/null
    bind enter __terminal_assistant_on_enter 2>/dev/null
    bind -M insert enter __terminal_assistant_on_enter 2>/dev/null

    # Cancel commandline (Ctrl+C)
    bind \cC __terminal_assistant_on_cancel
    bind -M insert \cC __terminal_assistant_on_cancel 2>/dev/null

    # Manual trigger fallback (Ctrl+Space / \c@)
    bind \c@ __terminal_assistant_manual_suggest
    bind -M insert \c@ __terminal_assistant_manual_suggest 2>/dev/null
end

__terminal_assistant_bind_keys
