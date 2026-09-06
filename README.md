<div align="center">

# ⚡ ShellSense

**IntelliSense-like terminal command assistant and autocomplete layer for Linux.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Linux%20(Any%20Distro)-green.svg)](https://github.com/Prithibi17/ShellSense)
[![Shells](https://img.shields.io/badge/Shells-Fish%20%7C%20Bash%20%7C%20Zsh-purple.svg)](https://github.com/Prithibi17/ShellSense)
[![Latency](https://img.shields.io/badge/Latency-%3C%201ms%20(Native)-brightgreen.svg)](https://github.com/Prithibi17/ShellSense)

<p align="center">
  Transform natural intent into verified commands directly in your prompt.<br>
  <b>Zero cloud latency. 100% offline. Zero keyboard lag.</b>
</p>

```
prithibi@cachyos ~> instal chrome  [Hit Tab]
prithibi@cachyos ~> paru -S google-chrome

prithibi@cachyos ~> check gpu  [Hit Tab]
prithibi@cachyos ~> nvidia-smi

prithibi@cachyos ~> restart audio  [Hit Tab]
prithibi@cachyos ~> systemctl --user restart pipewire wireplumber
```

</div>

---

## ✨ Features

- ⚡ **Sub-Millisecond Latency (< 1ms)**:
  - Direct zero-overhead hardware and distro detection using Linux `/sys` and `/proc` filesystems.
  - Native Rust daemon communicating over lightning-fast Unix domain sockets (`$XDG_RUNTIME_DIR/shellsense.sock`).

- 🛡️ **Safe Typing Guarantee**:
  - **Zero prompt lag or freezing**. Printable keystrokes are never intercepted, clobbered, or delayed.
  - Autocomplete attaches cleanly to standard shell completion triggers (<kbd>Tab</kbd>, <kbd>→</kbd>).

- 🧠 **100% Offline & Deterministic**:
  - **No AI required** by default. Runs entirely offline with zero GPU/RAM overhead (~12MB memory).
  - 110+ popular software package mappings (`chrome`, `discord`, `vscode`, `spotify`, `steam`, `docker`, `neovim`, etc.).
  - Context & path awareness: detects file types in the current directory (`extract archive` -> `unzip archive.zip`, `run project` -> `cargo run` / `npm run dev`).
  - Git repository heuristics (`save changes` -> `git add .`, `undo commit` -> `git reset --soft HEAD~1`).

- 🐧 **Universal Linux Support (Any Distro & Hardware)**:
  - **Distros**: Arch / CachyOS (`paru`/`pacman`), Debian / Ubuntu (`apt`), Fedora / RHEL (`dnf`), openSUSE (`zypper`), Alpine (`apk`), Void (`xbps`).
  - **GPUs**: Automatically detects NVIDIA (`nvidia-smi`), AMD Radeon (`radeontop`/`rocm-smi`), Intel Arc/iGPU (`intel_gpu_top`), or fallback `lspci`.
  - **Laptops & Power**: FreeDesktop universal standards (`powerprofilesctl`, `brightnessctl`, `upower`), plus hardware-specific tools like ASUS TUF/ROG (`asusctl`).
  - **Audio**: PipeWire, WirePlumber, PulseAudio, and ALSA.

- 🐚 **Native Shell Integrations**:
  - First-class support for **Fish**, **Bash**, and **Zsh**.

- 🤖 **Optional AI Intent Engine**:
  - Need natural language synthesis for obscure commands? Turn on local **Ollama** in `config.toml` (`ai_enabled = true`).

- ⚠️ **Safety & Destructive Command Warnings**:
  - Commands like `rm -rf`, `mkfs`, `dd of=/dev/`, `fdisk`, and `wipefs` are flagged with destructive warnings. Commands are **never auto-executed**—they are placed into the line buffer for your review.

---

## 🚀 Quick Install

### One-Line Automated Installer (Recommended)

Works on any Linux distribution (Fish, Bash, Zsh):

```bash
git clone https://github.com/Prithibi17/ShellSense.git
cd shellsense
./install.sh
```

The installer will:
1. Compile the release binaries (`shellsense`, `shellsensd`).
2. Install them to `~/.local/bin/` with a short alias `ss`.
3. Configure integrations for your installed shells (**Fish**, **Bash**, **Zsh**).
4. Start and enable the systemd user service (`shellsense.service`).

---

### Arch Linux / CachyOS (PKGBUILD)

```bash
git clone https://github.com/Prithibi17/ShellSense.git
cd shellsense
makepkg -si
```

---

### Manual Cargo Installation

```bash
cargo build --release
mkdir -p ~/.local/bin ~/.config/shellsense
cp target/release/shellsense target/release/shellsensd ~/.local/bin/
ln -sf ~/.local/bin/shellsense ~/.local/bin/ss
cp config/config.toml ~/.config/shellsense/
```

Then install the shell integration for your shell:
- **Fish**: `cp fish/shellsense.fish ~/.config/fish/conf.d/`
- **Bash**: Add `source ~/.config/shellsense/shellsense.bash` to `~/.bashrc`
- **Zsh**: Add `source ~/.config/shellsense/shellsense.zsh` to `~/.zshrc`

---

## ⌨️ How to Use

Simply type natural intent or shorthand in your terminal prompt:

| Keystroke | Action |
| :--- | :--- |
| <kbd>Tab</kbd> or <kbd>→</kbd> (Right Arrow) | Accept suggestion and replace buffer |
| <kbd>Tab</kbd> (consecutive) / <kbd>Alt</kbd> + <kbd>↓</kbd> | Cycle through alternative candidates |
| <kbd>Esc</kbd> | Revert buffer to originally typed text |
| Standard Keys | Normal typing with zero lag or interference |

### Example Commands

| You Type | Hit <kbd>Tab</kbd> | ShellSense Suggestion |
| :--- | :--- | :--- |
| `instal chrome` | ⇥ | `paru -S google-chrome` (Arch) / `sudo apt install google-chrome-stable` (Debian) |
| `check gpu` | ⇥ | `nvidia-smi` *(on NVIDIA)* or `radeontop` *(on AMD)* |
| `restart audio` | ⇥ | `systemctl --user restart pipewire wireplumber` |
| `show ip` | ⇥ | `ip --brief address` |
| `clean orphans` | ⇥ | `paru -Rns (pacman -Qtdq)` |
| `battery health` | ⇥ | `upower -i /org/freedesktop/UPower/devices/battery_BAT0` |
| `extract archive` | ⇥ | `unzip "my_file.zip"` *(context-aware: detects `.zip` in cwd)* |
| `run project` | ⇥ | `cargo run` *(context-aware: detects `Cargo.toml` in cwd)* |

---

## 💻 CLI & Alias Usage

ShellSense provides the `shellsense` command and a short alias `ss`:

```bash
# Query suggestion from the command line
ss suggest "instal discord"

# Print raw command (perfect for scripts)
ss suggest --raw "check memory"

# View daemon status and auto-detected hardware
ss status

# Explain a command
ss explain "sudo pacman -Syu"
```

---

## ⚙️ Configuration

Configuration is located at `~/.config/shellsense/config.toml`:

```toml
# ShellSense Configuration

# Set to true to enable local Ollama AI synthesis for unknown commands
ai_enabled = false

# Ollama local model and endpoint
model = "deepseek-r1:1.5b"
ollama_url = "http://localhost:11434"

# Cache time-to-live in seconds
cache_ttl_secs = 3600
```

---

## 🔧 Managing the Background Service

ShellSense runs a lightweight daemon in user space:

```bash
# Check status
systemctl --user status shellsense.service

# Restart service
systemctl --user restart shellsense.service

# View daemon logs
journalctl --user -u shellsense.service -f
```

---

## 🏗️ Architecture

```
                    Terminal Shell (Fish, Bash, or Zsh)
                [Normal typing: 100% untouched & unblocked]
                                    │
                         User hits <Tab> or <→>
                                    │
                                    ▼
                 $XDG_RUNTIME_DIR/shellsense.sock
                                    │  (< 0.1ms Unix socket)
                                    ▼
                    ┌───────────────────────────────┐
                    │          shellsensd           │
                    ├───────────────────────────────┤
                    │ 1. Zero-Overhead Sysfs Sensor │
                    │    (Distro, GPU, Audio, Battery)
                    │ 2. Deterministic Intent Engine│
                    │ 3. Path & Workspace Context   │
                    │ 4. Distro Syntax Normalizer   │
                    │ 5. Safety Classifier          │
                    │ 6. (Optional) Ollama AI       │
                    └───────────────────────────────┘
```

---

## 🧪 Testing

ShellSense includes an automated unit and integration test suite:

```bash
cargo test
```

All 17 tests run in sub-second time verifying package mappings, GPU heuristics, context awareness, and destructive command classification.

---

## 🗑️ Uninstallation

To cleanly remove ShellSense:

```bash
./scripts/uninstall.sh
```

This will stop the daemon, remove the systemd user service, remove the binaries and aliases, and clean up shell integrations.

---

## 📄 License

Licensed under the [MIT License](LICENSE).
