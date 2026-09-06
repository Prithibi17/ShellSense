#!/usr/bin/env bash
# install.sh - Universal Linux Installer for ShellSense
# Works on any Linux distribution (Arch, Debian, Ubuntu, Fedora, openSUSE, Alpine, Void, etc.)
# Supports Fish, Bash, and Zsh on any hardware configuration.

set -e

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}=== Installing ShellSense (Universal Linux) ===${RESET}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/shellsense"

mkdir -p "$BIN_DIR"
mkdir -p "$CONFIG_DIR"

# 1. Stop legacy services if running
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop terminal-assistant.service 2>/dev/null || true
    systemctl --user disable terminal-assistant.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/terminal-assistant.service"
    systemctl --user stop shellsense.service 2>/dev/null || true
fi

# Remove legacy fish integration if present
rm -f "$HOME/.config/fish/conf.d/terminal-assistant.fish"

# 2. Build or locate release binaries
if [[ -f "$SCRIPT_DIR/target/release/shellsense" && -f "$SCRIPT_DIR/target/release/shellsensd" ]]; then
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

# 3. Install binaries to ~/.local/bin and symlink 'ss'
echo -e "${CYAN}Installing binaries to ${BIN_DIR}...${RESET}"
cp -f "$SCRIPT_DIR/target/release/shellsense" "$BIN_DIR/shellsense"
cp -f "$SCRIPT_DIR/target/release/shellsensd" "$BIN_DIR/shellsensd"
chmod +x "$BIN_DIR/shellsense" "$BIN_DIR/shellsensd"

# Create short alias 'ss' -> shellsense if not clashing
ln -sf "$BIN_DIR/shellsense" "$BIN_DIR/ss"

# Also copy to ~/.cargo/bin if present
if [[ -d "$HOME/.cargo/bin" ]]; then
    cp -f "$BIN_DIR/shellsense" "$HOME/.cargo/bin/" 2>/dev/null || true
    cp -f "$BIN_DIR/shellsensd" "$HOME/.cargo/bin/" 2>/dev/null || true
    ln -sf "$HOME/.cargo/bin/shellsense" "$HOME/.cargo/bin/ss" 2>/dev/null || true
fi

# 4. Create default configuration if not present (or copy from legacy)
if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    if [[ -f "$HOME/.config/terminal-assistant/config.toml" ]]; then
        cp "$HOME/.config/terminal-assistant/config.toml" "$CONFIG_DIR/config.toml"
        echo -e "${GREEN}✓ Migrated existing configuration to ${CONFIG_DIR}/config.toml${RESET}"
    else
        cat << 'EOF' > "$CONFIG_DIR/config.toml"
# ShellSense Configuration
ai_enabled = false
model = "deepseek-r1:1.5b"
ollama_url = "http://localhost:11434"
cache_ttl_secs = 3600
EOF
        echo -e "${GREEN}✓ Created default config at ${CONFIG_DIR}/config.toml${RESET}"
    fi
fi

# 5. Configure Shell Integrations (Fish, Bash, Zsh)
echo -e "${CYAN}Configuring shell integrations...${RESET}"

# Fish shell
if command -v fish >/dev/null 2>&1 || [[ -d "$HOME/.config/fish" ]]; then
    mkdir -p "$HOME/.config/fish/conf.d"
    cp -f "$SCRIPT_DIR/fish/shellsense.fish" "$HOME/.config/fish/conf.d/shellsense.fish"
    echo -e "${GREEN}✓ Fish integration installed to ~/.config/fish/conf.d/shellsense.fish${RESET}"
fi

# Bash shell
BASH_LINE="[ -f \"$CONFIG_DIR/shellsense.bash\" ] && source \"$CONFIG_DIR/shellsense.bash\""
cp -f "$SCRIPT_DIR/bash/shellsense.bash" "$CONFIG_DIR/shellsense.bash"
if [[ -f "$HOME/.bashrc" ]]; then
    # Remove any old terminal-assistant references
    sed -i '/terminal-assistant\.bash/d' "$HOME/.bashrc" 2>/dev/null || true
    if ! grep -q "shellsense.bash" "$HOME/.bashrc"; then
        echo -e "\n# ShellSense\n$BASH_LINE" >> "$HOME/.bashrc"
    fi
    echo -e "${GREEN}✓ Bash integration enabled in ~/.bashrc${RESET}"
fi

# Zsh shell
ZSH_LINE="[ -f \"$CONFIG_DIR/shellsense.zsh\" ] && source \"$CONFIG_DIR/shellsense.zsh\""
cp -f "$SCRIPT_DIR/zsh/shellsense.zsh" "$CONFIG_DIR/shellsense.zsh"
if [[ -f "$HOME/.zshrc" ]] || command -v zsh >/dev/null 2>&1; then
    touch "$HOME/.zshrc"
    sed -i '/terminal-assistant\.zsh/d' "$HOME/.zshrc" 2>/dev/null || true
    if ! grep -q "shellsense.zsh" "$HOME/.zshrc"; then
        echo -e "\n# ShellSense\n$ZSH_LINE" >> "$HOME/.zshrc"
    fi
    echo -e "${GREEN}✓ Zsh integration enabled in ~/.zshrc${RESET}"
fi

# 6. Service configuration (Systemd user service or background daemon)
if command -v systemctl >/dev/null 2>&1; then
    SYSTEMD_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_DIR"
    cat << EOF > "$SYSTEMD_DIR/shellsense.service"
[Unit]
Description=ShellSense Intelligent Command Autocomplete Daemon
After=network.target

[Service]
Type=simple
ExecStart=$BIN_DIR/shellsensd
Restart=always
RestartSec=2
Environment=PATH=$BIN_DIR:$PATH

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload
    systemctl --user enable --now shellsense.service
    echo -e "${GREEN}✓ Systemd user service shellsense.service started and enabled${RESET}"
else
    echo -e "${YELLOW}Starting background daemon...${RESET}"
    pkill -f "shellsensd" 2>/dev/null || true
    nohup "$BIN_DIR/shellsensd" >/dev/null 2>&1 &
    echo -e "${GREEN}✓ Background daemon started${RESET}"
fi

# 7. Verification
echo ""
echo -e "${BOLD}${GREEN}=== ShellSense Successfully Installed! ===${RESET}"
echo -e "Testing CLI: 'instal chrome' -> ${CYAN}$("$BIN_DIR/shellsense" suggest --raw "instal chrome")${RESET}"
echo -e "Short command: ${CYAN}ss${RESET} or ${CYAN}shellsense${RESET}"
echo ""
echo -e "Usage:"
echo -e "  - Open a new terminal tab or reload your shell."
echo -e "  - Type an intent like ${CYAN}instal chrome${RESET} or ${CYAN}check gpu${RESET} and press ${BOLD}Tab${RESET} or ${BOLD}Right Arrow${RESET}."
echo -e "  - Press ${BOLD}Tab${RESET} again to cycle candidates, or ${BOLD}Esc${RESET} to revert."
