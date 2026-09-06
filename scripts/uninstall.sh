#!/usr/bin/env bash
# scripts/uninstall.sh — One-command ShellSense uninstaller
# Run directly:  bash <(curl -fsSL https://raw.githubusercontent.com/Prithibi17/ShellSense/main/scripts/uninstall.sh)
# Or locally:    ./scripts/uninstall.sh

GREEN="\033[32m"; CYAN="\033[36m"; YELLOW="\033[33m"; BOLD="\033[1m"; RESET="\033[0m"

echo -e "${BOLD}${CYAN}=== ShellSense Uninstaller ===${RESET}"

# 1. Stop + disable daemon
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user stop    shellsense.service 2>/dev/null || true
    systemctl --user disable shellsense.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/shellsense.service"
    systemctl --user daemon-reload 2>/dev/null || true
fi
pkill -f shellsensd 2>/dev/null || true
echo -e "${GREEN}✓ Daemon stopped${RESET}"

# 2. Remove binaries
for dir in "$HOME/.local/bin" "$HOME/.cargo/bin" "/usr/local/bin"; do
    rm -f "$dir/shellsense" "$dir/shellsensd" "$dir/ss" 2>/dev/null || true
done
echo -e "${GREEN}✓ Binaries removed${RESET}"

# 3. Remove fish integration
rm -f "$HOME/.config/fish/conf.d/shellsense.fish"

# 4. Remove bash/zsh hooks
for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] && sed -i '/[Ss]hell[Ss]ense\|shellsense/d' "$rc" 2>/dev/null || true
done
echo -e "${GREEN}✓ Shell integrations removed${RESET}"

# 5. Remove runtime socket
rm -f "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/shellsense.sock"

# 6. Ask about config + data
echo ""
read -rp "Also delete config + learning history? (~/.config/shellsense) [y/N]: " ans
if [[ "${ans,,}" == "y" ]]; then
    rm -rf "$HOME/.config/shellsense"
    rm -rf "$HOME/.local/share/shellsense"
    echo -e "${GREEN}✓ Config and data removed${RESET}"
else
    echo -e "${YELLOW}  Config kept at ~/.config/shellsense${RESET}"
fi

echo ""
echo -e "${BOLD}${GREEN}✓ ShellSense completely uninstalled.${RESET}"
echo -e "  Open a new terminal tab and ShellSense will no longer be active."
