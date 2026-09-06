#!/usr/bin/env bash
# install.sh - Universal Linux Installer for Terminal Assistant
# Works on any Linux distribution (Arch, Debian, Ubuntu, Fedora, openSUSE, Alpine, Void, etc.)
# Supports Fish, Bash, and Zsh on any hardware configuration.

set -e

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}=== Installing Terminal Assistant (Universal Linux) ===${RESET}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/terminal-assistant"

mkdir -p "$BIN_DIR"
mkdir -p "$CONFIG_DIR"

# 1. Build or locate release binaries
if [[ -f "$SCRIPT_DIR/target/release/terminal-assistant" && -f "$SCRIPT_DIR/target/release/terminal-assistantd" ]]; then
    echo -e "${GREEN}✓ Found compiled release binaries${RESET}"
else
    echo -e "${CYAN}Compiling release binaries using cargo...${RESET}"
    if command -v cargo >/dev/null 2>&1; then
        (cd "$SCRIPT_DIR" && cargo build --release)
    else
        echo -e "${YELLOW}Error: cargo not found. Please install Rust or precompile release binaries.${RESET}"
        exit 1
    fi
fi

# 2. Install binaries to ~/.local/bin
echo -e "${CYAN}Installing binaries to ${BIN_DIR}...${RESET}"

# If systemd service is active, stop it briefly during copy
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop terminal-assistant.service 2>/dev/null || true
fi

cp -f "$SCRIPT_DIR/target/release/terminal-assistant" "$BIN_DIR/terminal-assistant"
cp -f "$SCRIPT_DIR/target/release/terminal-assistantd" "$BIN_DIR/terminal-assistantd"
chmod +x "$BIN_DIR/terminal-assistant" "$BIN_DIR/terminal-assistantd"

# Also copy to ~/.cargo/bin if directory exists
if [[ -d "$HOME/.cargo/bin" ]]; then
    cp -f "$BIN_DIR/terminal-assistant" "$HOME/.cargo/bin/" 2>/dev/null || true
    cp -f "$BIN_DIR/terminal-assistantd" "$HOME/.cargo/bin/" 2>/dev/null || true
fi

# 3. Create default configuration if not present
if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    cat << 'EOF' > "$CONFIG_DIR/config.toml"
# Terminal Assistant Configuration
ai_enabled = false
model = "deepseek-r1:1.5b"
ollama_url = "http://localhost:11434"
cache_ttl_secs = 3600
EOF
    echo -e "${GREEN}✓ Created default config at ${CONFIG_DIR}/config.toml${RESET}"
fi

# 4. Configure Shell Integrations (Fish, Bash, Zsh)
echo -e "${CYAN}Configuring shell integrations...${RESET}"

# Fish shell
if command -v fish >/dev/null 2>&1 || [[ -d "$HOME/.config/fish" ]]; then
    mkdir -p "$HOME/.config/fish/conf.d"
    cp -f "$SCRIPT_DIR/fish/terminal-assistant.fish" "$HOME/.config/fish/conf.d/terminal-assistant.fish"
    echo -e "${GREEN}✓ Fish integration installed to ~/.config/fish/conf.d/terminal-assistant.fish${RESET}"
fi

# Bash shell
BASH_LINE="[ -f \"$CONFIG_DIR/terminal-assistant.bash\" ] && source \"$CONFIG_DIR/terminal-assistant.bash\""
cp -f "$SCRIPT_DIR/bash/terminal-assistant.bash" "$CONFIG_DIR/terminal-assistant.bash"
if [[ -f "$HOME/.bashrc" ]]; then
    if ! grep -q "terminal-assistant.bash" "$HOME/.bashrc"; then
        echo -e "\n# Terminal Assistant\n$BASH_LINE" >> "$HOME/.bashrc"
    fi
    echo -e "${GREEN}✓ Bash integration enabled in ~/.bashrc${RESET}"
fi

# Zsh shell
ZSH_LINE="[ -f \"$CONFIG_DIR/terminal-assistant.zsh\" ] && source \"$CONFIG_DIR/terminal-assistant.zsh\""
cp -f "$SCRIPT_DIR/zsh/terminal-assistant.zsh" "$CONFIG_DIR/terminal-assistant.zsh"
if [[ -f "$HOME/.zshrc" ]] || command -v zsh >/dev/null 2>&1; then
    touch "$HOME/.zshrc"
    if ! grep -q "terminal-assistant.zsh" "$HOME/.zshrc"; then
        echo -e "\n# Terminal Assistant\n$ZSH_LINE" >> "$HOME/.zshrc"
    fi
    echo -e "${GREEN}✓ Zsh integration enabled in ~/.zshrc${RESET}"
fi

# 5. Service configuration (Systemd user service or background daemon)
if command -v systemctl >/dev/null 2>&1; then
    SYSTEMD_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_DIR"
    cat << EOF > "$SYSTEMD_DIR/terminal-assistant.service"
[Unit]
Description=Terminal Assistant AI Autocomplete Daemon
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/terminal-assistantd
Restart=always
RestartSec=2
Environment=PATH=$BIN_DIR:$PATH

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload
    systemctl --user enable --now terminal-assistant.service
    echo -e "${GREEN}✓ Systemd user service started and enabled${RESET}"
else
    # Non-systemd system (Alpine, Void, OpenRC)
    echo -e "${YELLOW}Starting background daemon...${RESET}"
    pkill -f "terminal-assistantd" 2>/dev/null || true
    nohup "$BIN_DIR/terminal-assistantd" >/dev/null 2>&1 &
    echo -e "${GREEN}✓ Background daemon started${RESET}"
fi

# 6. Verify installation
echo ""
echo -e "${BOLD}${GREEN}=== Terminal Assistant Successfully Installed! ===${RESET}"
echo -e "Testing assistant: 'instal chrome' -> ${CYAN}$("$BIN_DIR/terminal-assistant" suggest --raw "instal chrome")${RESET}"
echo ""
echo -e "Usage:"
echo -e "  - Open a new terminal or reload your shell."
echo -e "  - Type an intent like ${CYAN}instal chrome${RESET} or ${CYAN}check gpu${RESET} and press ${BOLD}Tab${RESET} or ${BOLD}Right Arrow${RESET}."
echo -e "  - Press ${BOLD}Tab${RESET} again to cycle candidates, or ${BOLD}Esc${RESET} to revert."
