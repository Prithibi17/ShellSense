#!/usr/bin/env bash
# install.sh - Universal 1-Line Installer for ShellSense
# Can be run via:
#   curl -fsSL https://raw.githubusercontent.com/Prithibi17/ShellSense/main/install.sh | bash
# or locally inside a cloned repository:
#   ./install.sh

set -e

REPO="Prithibi17/ShellSense"
RAW_URL="https://raw.githubusercontent.com/${REPO}/main"
BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/shellsense"

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}=== ShellSense Universal Installer ===${RESET}"

# Verify OS
OS="$(uname -s)"
if [[ "$OS" != "Linux" ]]; then
    echo -e "${RED}Error: ShellSense currently supports Linux. Detected: $OS${RESET}"
    exit 1
fi

ARCH="$(uname -m)"

mkdir -p "$BIN_DIR"
mkdir -p "$CONFIG_DIR"

# Determine script context (local repository vs piped from curl)
LOCAL_REPO=0
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd || echo "")"
if [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" && -f "$SCRIPT_DIR/fish/shellsense.fish" ]]; then
    LOCAL_REPO=1
fi

# 1. Stop existing daemon during upgrade
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop shellsense.service 2>/dev/null || true
fi
pkill -f "shellsensd" 2>/dev/null || true

# 2. Obtain release binaries (Pre-built release -> Local compiled -> Cargo build)
INSTALLED_BINARIES=0

# Option A: Local pre-compiled binaries in target/release/
if [[ "$LOCAL_REPO" -eq 1 && -f "$SCRIPT_DIR/target/release/shellsense" && -f "$SCRIPT_DIR/target/release/shellsensd" ]]; then
    echo -e "${GREEN}✓ Using local compiled binaries${RESET}"
    cp -f "$SCRIPT_DIR/target/release/shellsense" "$BIN_DIR/shellsense"
    cp -f "$SCRIPT_DIR/target/release/shellsensd" "$BIN_DIR/shellsensd"
    INSTALLED_BINARIES=1
fi

# Option B: Download pre-built release binary from GitHub Releases
if [[ "$INSTALLED_BINARIES" -eq 0 && "$ARCH" == "x86_64" ]]; then
    echo -e "${CYAN}Checking for pre-compiled GitHub Release binary...${RESET}"
    RELEASE_URL="https://github.com/${REPO}/releases/latest/download/shellsense-linux-x86_64.tar.gz"
    TMP_DIR="$(mktemp -d)"
    if curl -fsSL --connect-timeout 5 --max-time 20 "$RELEASE_URL" -o "$TMP_DIR/shellsense.tar.gz" 2>/dev/null; then
        echo -e "${GREEN}✓ Downloaded pre-compiled binary release${RESET}"
        tar -xzf "$TMP_DIR/shellsense.tar.gz" -C "$TMP_DIR"
        if [[ -f "$TMP_DIR/shellsense" && -f "$TMP_DIR/shellsensd" ]]; then
            cp -f "$TMP_DIR/shellsense" "$BIN_DIR/shellsense"
            cp -f "$TMP_DIR/shellsensd" "$BIN_DIR/shellsensd"
            INSTALLED_BINARIES=1
        fi
    fi
    rm -rf "$TMP_DIR"
fi

# Option C: Compile from source using cargo
if [[ "$INSTALLED_BINARIES" -eq 0 ]]; then
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${CYAN}Compiling ShellSense from source using cargo...${RESET}"
        if [[ "$LOCAL_REPO" -eq 1 ]]; then
            (cd "$SCRIPT_DIR" && cargo build --release)
            cp -f "$SCRIPT_DIR/target/release/shellsense" "$BIN_DIR/shellsense"
            cp -f "$SCRIPT_DIR/target/release/shellsensd" "$BIN_DIR/shellsensd"
            INSTALLED_BINARIES=1
        else
            TMP_BUILD="$(mktemp -d)"
            if command -v git >/dev/null 2>&1; then
                git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_BUILD/shellsense"
                (cd "$TMP_BUILD/shellsense" && cargo build --release)
                cp -f "$TMP_BUILD/shellsense/target/release/shellsense" "$BIN_DIR/shellsense"
                cp -f "$TMP_BUILD/shellsense/target/release/shellsensd" "$BIN_DIR/shellsensd"
                INSTALLED_BINARIES=1
            fi
            rm -rf "$TMP_BUILD"
        fi
    fi
fi

if [[ "$INSTALLED_BINARIES" -eq 0 ]]; then
    echo -e "${RED}Error: Could not obtain ShellSense binaries.${RESET}"
    echo -e "Please ensure internet connection, or install Rust with 'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh'."
    exit 1
fi

chmod +x "$BIN_DIR/shellsense" "$BIN_DIR/shellsensd"
ln -sf "$BIN_DIR/shellsense" "$BIN_DIR/ss"

# Also sync to ~/.cargo/bin if user has it
if [[ -d "$HOME/.cargo/bin" ]]; then
    cp -f "$BIN_DIR/shellsense" "$HOME/.cargo/bin/" 2>/dev/null || true
    cp -f "$BIN_DIR/shellsensd" "$HOME/.cargo/bin/" 2>/dev/null || true
    ln -sf "$HOME/.cargo/bin/shellsense" "$HOME/.cargo/bin/ss" 2>/dev/null || true
fi

# Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${YELLOW}Notice: Adding ${BIN_DIR} to your PATH${RESET}"
    export PATH="$BIN_DIR:$PATH"
fi

# 3. Setup Default Configuration
if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    cat << 'EOF' > "$CONFIG_DIR/config.toml"
# ShellSense Configuration
ai_enabled = false
model = "deepseek-r1:1.5b"
ollama_url = "http://localhost:11434"
cache_ttl_secs = 3600
EOF
    echo -e "${GREEN}✓ Created configuration at ${CONFIG_DIR}/config.toml${RESET}"
fi

# 4. Configure Shell Integrations (Fish, Bash, Zsh)
echo -e "${CYAN}Configuring shell integrations...${RESET}"

install_shell_file() {
    local rel_path="$1"
    local dest="$2"
    mkdir -p "$(dirname "$dest")"
    if [[ "$LOCAL_REPO" -eq 1 && -f "$SCRIPT_DIR/$rel_path" ]]; then
        cp -f "$SCRIPT_DIR/$rel_path" "$dest"
    else
        curl -fsSL --connect-timeout 5 "${RAW_URL}/${rel_path}" -o "$dest" 2>/dev/null || true
    fi
}

# Fish Shell
if command -v fish >/dev/null 2>&1 || [[ -d "$HOME/.config/fish" ]]; then
    install_shell_file "fish/shellsense.fish" "$HOME/.config/fish/conf.d/shellsense.fish"
    echo -e "${GREEN}✓ Fish integration enabled (~/.config/fish/conf.d/shellsense.fish)${RESET}"
fi

# Bash Shell
BASH_SCRIPT="$CONFIG_DIR/shellsense.bash"
install_shell_file "bash/shellsense.bash" "$BASH_SCRIPT"
if [[ -f "$HOME/.bashrc" ]] || command -v bash >/dev/null 2>&1; then
    touch "$HOME/.bashrc"
    sed -i '/shellsense\.bash/d' "$HOME/.bashrc" 2>/dev/null || true
    sed -i '/# ShellSense/d' "$HOME/.bashrc" 2>/dev/null || true
    echo -e "\n# ShellSense\n[ -f \"$BASH_SCRIPT\" ] && source \"$BASH_SCRIPT\"" >> "$HOME/.bashrc"
    echo -e "${GREEN}✓ Bash integration enabled (~/.bashrc)${RESET}"
fi

# Zsh Shell
ZSH_SCRIPT="$CONFIG_DIR/shellsense.zsh"
install_shell_file "zsh/shellsense.zsh" "$ZSH_SCRIPT"
if [[ -f "$HOME/.zshrc" ]] || command -v zsh >/dev/null 2>&1; then
    touch "$HOME/.zshrc"
    sed -i '/shellsense\.zsh/d' "$HOME/.zshrc" 2>/dev/null || true
    sed -i '/# ShellSense/d' "$HOME/.zshrc" 2>/dev/null || true
    echo -e "\n# ShellSense\n[ -f \"$ZSH_SCRIPT\" ] && source \"$ZSH_SCRIPT\"" >> "$HOME/.zshrc"
    echo -e "${GREEN}✓ Zsh integration enabled (~/.zshrc)${RESET}"
fi

# 5. Background Daemon / Service
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
    systemctl --user enable --now shellsense.service >/dev/null 2>&1 || true
    echo -e "${GREEN}✓ Systemd user service enabled and started${RESET}"
else
    nohup "$BIN_DIR/shellsensd" >/dev/null 2>&1 &
    echo -e "${GREEN}✓ Background daemon started${RESET}"
fi

# 6. Verification
echo ""
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo -e "${BOLD}${GREEN}  ShellSense Successfully Installed! 🚀${RESET}"
echo -e "${BOLD}${GREEN}====================================================${RESET}"
echo -e "Testing: ${CYAN}$("$BIN_DIR/shellsense" suggest --raw "instal chrome")${RESET}"
echo ""
echo -e "How to use:"
echo -e "  1. Open a new terminal tab (or reload your shell)."
echo -e "  2. Type any intent: ${CYAN}instal chrome${RESET}, ${CYAN}check gpu${RESET}, ${CYAN}restart audio${RESET}"
echo -e "  3. ShellSense displays a ghost preview ${CYAN}→ command${RESET} automatically!"
echo -e "  4. Hit ${BOLD}Tab${RESET} or ${BOLD}Enter${RESET} to run."
echo ""
echo -e "Short CLI command: ${CYAN}ss${RESET} (e.g. ${BOLD}ss status${RESET}, ${BOLD}ss suggest \"intent\"${RESET})"
