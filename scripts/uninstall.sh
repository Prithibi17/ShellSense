#!/usr/bin/env bash
# scripts/uninstall.sh - Clean uninstaller for ShellSense
set -e

RED="\033[31m"
GREEN="\033[32m"
CYAN="\033[36m"
RESET="\033[0m"

echo -e "${CYAN}==> Uninstalling ShellSense...${RESET}"

# 1. Stop and disable systemd service
if command -v systemctl >/dev/null 2>&1; then
    echo "Stopping systemd user service..."
    systemctl --user stop shellsense.service 2>/dev/null || true
    systemctl --user disable shellsense.service 2>/dev/null || true
    rm -f "${HOME}/.config/systemd/user/shellsense.service"
    systemctl --user daemon-reload 2>/dev/null || true
fi

# Stop any running process directly
pkill -f "shellsensd" 2>/dev/null || true

# 2. Remove shell integrations
echo "Removing shell integrations..."
rm -f "${HOME}/.config/fish/conf.d/shellsense.fish"
rm -f "${HOME}/.config/shellsense/shellsense.bash"
rm -f "${HOME}/.config/shellsense/shellsense.zsh"

if [[ -f "${HOME}/.bashrc" ]]; then
    sed -i '/shellsense\.bash/d' "${HOME}/.bashrc" 2>/dev/null || true
    sed -i '/# ShellSense/d' "${HOME}/.bashrc" 2>/dev/null || true
fi

if [[ -f "${HOME}/.zshrc" ]]; then
    sed -i '/shellsense\.zsh/d' "${HOME}/.zshrc" 2>/dev/null || true
    sed -i '/# ShellSense/d' "${HOME}/.zshrc" 2>/dev/null || true
fi

# 3. Remove binaries and symlinks
echo "Removing binaries..."
rm -f "${HOME}/.local/bin/shellsense"
rm -f "${HOME}/.local/bin/shellsensd"
rm -f "${HOME}/.local/bin/ss"

if [[ -d "${HOME}/.cargo/bin" ]]; then
    rm -f "${HOME}/.cargo/bin/shellsense"
    rm -f "${HOME}/.cargo/bin/shellsensd"
    rm -f "${HOME}/.cargo/bin/ss"
fi

# 4. Clean up runtime sockets
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
rm -f "${RUNTIME_DIR}/shellsense.sock"

echo -e "${GREEN}✓ ShellSense has been completely uninstalled.${RESET}"
echo -e "Note: Configuration at ${HOME}/.config/shellsense/ was preserved. To remove it completely:"
echo -e "  rm -rf \"${HOME}/.config/shellsense\""
