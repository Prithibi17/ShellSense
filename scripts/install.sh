#!/usr/bin/env bash
# install.sh - Installer for terminal-assistant on CachyOS / Arch Linux

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/terminal-assistant"
FISH_CONF_DIR="${HOME}/.config/fish/conf.d"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

echo "==> Building terminal-assistant (release mode)..."
cd "${SCRIPT_DIR}"
cargo build --release

echo "==> Installing binaries to ${BIN_DIR}..."
mkdir -p "${BIN_DIR}"
install -m 755 "${SCRIPT_DIR}/target/release/terminal-assistant" "${BIN_DIR}/terminal-assistant"
install -m 755 "${SCRIPT_DIR}/target/release/terminal-assistantd" "${BIN_DIR}/terminal-assistantd"

# Ensure ~/.local/bin is in PATH notice
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "Notice: ${BIN_DIR} is not in your PATH. Ensure your shell config adds it."
fi

echo "==> Setting up default configuration..."
mkdir -p "${CONFIG_DIR}"
if [[ ! -f "${CONFIG_DIR}/config.toml" ]]; then
    cp "${SCRIPT_DIR}/config/config.toml" "${CONFIG_DIR}/config.toml"
    echo "Created ${CONFIG_DIR}/config.toml"
else
    echo "Existing configuration preserved at ${CONFIG_DIR}/config.toml"
fi

echo "==> Installing Fish shell plugin..."
mkdir -p "${FISH_CONF_DIR}"
cp "${SCRIPT_DIR}/fish/terminal-assistant.fish" "${FISH_CONF_DIR}/terminal-assistant.fish"
echo "Installed fish plugin to ${FISH_CONF_DIR}/terminal-assistant.fish"

echo "==> Setting up systemd user service..."
mkdir -p "${SYSTEMD_USER_DIR}"
# Adjust ExecStart to point to the user binary location
cat << EOF > "${SYSTEMD_USER_DIR}/terminal-assistant.service"
[Unit]
Description=Terminal Assistant AI Autocomplete Daemon
After=default.target

[Service]
Type=simple
ExecStart=${BIN_DIR}/terminal-assistantd
Restart=on-failure
RestartSec=2
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now terminal-assistant.service || {
    echo "Notice: systemd user service could not be started immediately. You can start the daemon manually using 'terminal-assistantd'."
}

echo ""
echo "============================================================"
echo "  terminal-assistant successfully installed!"
echo "============================================================"
echo "  • Daemon:   ${BIN_DIR}/terminal-assistantd"
echo "  • CLI:      ${BIN_DIR}/terminal-assistant"
echo "  • Fish:     ${FISH_CONF_DIR}/terminal-assistant.fish"
echo "  • Config:   ${CONFIG_DIR}/config.toml"
echo ""
echo "To use right away in your current fish session:"
echo "  source ${FISH_CONF_DIR}/terminal-assistant.fish"
echo ""
echo "Try typing: 'install chrome' and press Ctrl+Space!"
echo "============================================================"
