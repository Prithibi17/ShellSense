#!/usr/bin/env bash
# uninstall.sh - Clean uninstaller for terminal-assistant

set -e

BIN_DIR="${HOME}/.local/bin"
FISH_CONF="${HOME}/.config/fish/conf.d/terminal-assistant.fish"
SYSTEMD_USER="${HOME}/.config/systemd/user/terminal-assistant.service"

echo "==> Stopping systemd user service..."
systemctl --user stop terminal-assistant.service 2>/dev/null || true
systemctl --user disable terminal-assistant.service 2>/dev/null || true
rm -f "${SYSTEMD_USER}"
systemctl --user daemon-reload

echo "==> Removing fish integration..."
rm -f "${FISH_CONF}"

echo "==> Removing binaries..."
rm -f "${BIN_DIR}/terminal-assistant"
rm -f "${BIN_DIR}/terminal-assistantd"

echo "==> Cleaning up runtime sockets..."
rm -f "${XDG_RUNTIME_DIR}/terminal-assistant.sock"

echo "terminal-assistant successfully uninstalled."
echo "Note: Configuration at ~/.config/terminal-assistant/ has been kept. Remove manually if desired."
