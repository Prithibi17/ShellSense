# Terminal Assistant (`terminal-assistant`)

A fast, native Linux terminal command assistant and autocomplete daemon engineered for **CachyOS (Arch Linux)**, **Hyprland**, and **fish shell**.

Combines **instant local deterministic completions** (< 10ms) with a local **Ollama AI intent engine** to seamlessly transform natural-language instructions into precise, verified Linux commands right at your terminal prompt.

```
prithibi@cachyos ~> instal chrome
                    └─ paru -S google-chrome
```

---

## Key Features

- **Blazing Fast**:
  - Deterministic completions & common commands: `< 10 ms`
  - Cached intent suggestions: `< 20 ms`
  - In-process fallback: shell commands never hang or fail even if the daemon is offline.
- **Tailored for CachyOS & Arch Linux**:
  - Prioritizes `pacman` and `paru` (AUR).
  - Automatically corrects Debian/Ubuntu commands (e.g., converts `apt install` to `sudo pacman -S`).
  - Native awareness of **Hyprland** (`hyprctl`), **PipeWire/WirePlumber** (`wpctl`, `systemctl --user`), **NVIDIA** (`nvidia-smi`), **Btrfs**, and **ASUS TUF/ROG** (`asusctl`).
- **Context & Path Awareness**:
  - Inspects current directory contents: typing `extract archive` when `archive.zip` exists suggests `unzip "archive.zip"`.
  - Project-aware: typing `run project` detects `Cargo.toml` (`cargo run`) or `package.json` (`npm run dev`).
  - Git repository aware: `save changes` -> `git add .`, `undo last commit` -> `git reset --soft HEAD~1`.
- **Destructive Command Safety Classifier**:
  - Commands like `rm -rf`, `mkfs`, `dd of=/dev/`, `pacman -Rns`, `wipefs`, `fdisk`, and `shutdown` are flagged as **`⚠ Potentially destructive`**.
  - **Zero Auto-Execution**: AI commands are only inserted into the terminal buffer; you must explicitly review and press <kbd>Enter</kbd>.
- **Privacy-First & Local**:
  - Context sanitizer automatically redacts API keys, bearer tokens, passwords, cookies, and private keys.
  - Zero cloud dependencies: utilizes local **Ollama** (`http://127.0.0.1:11434`).
- **Seamless Fish Integration**:
  - Press <kbd>Ctrl+Space</kbd> to transform natural language into a command in-place.
  - Ghost-text inline hints with <kbd>Tab</kbd> or <kbd>→</kbd> to accept and <kbd>Esc</kbd> to dismiss.

---

## Architecture

```
                                  Fish Shell
                          (commandline -r / -i)
                                     │
                             <Ctrl+Space> / Ghost
                                     │
                                     ▼
                     $XDG_RUNTIME_DIR/terminal-assistant.sock
                                     │
                                     ▼
                     ┌────────────────────────────────┐
                     │      terminal-assistantd       │
                     ├────────────────────────────────┤
                     │ 1. Local Learning Engine       │
                     │ 2. Deterministic Rules Engine  │
                     │ 3. Context & Path Analyzer     │
                     │ 4. Safety & Distro Validator   │
                     │ 5. Ollama AI Intent Engine     │
                     │ 6. In-Memory TTL Cache         │
                     └────────────────────────────────┘
```

---

## Installation

### Method 1: One-Step Script (Recommended)
```bash
git clone https://github.com/prithibi/terminal-assistant.git
cd terminal-assistant
./scripts/install.sh
```

### Method 2: Arch / CachyOS PKGBUILD
```bash
makepkg -si
# Or using paru
paru -B .
```

### Method 3: Cargo Install
```bash
cargo build --release
install -Dm755 target/release/terminal-assistant ~/.local/bin/terminal-assistant
install -Dm755 target/release/terminal-assistantd ~/.local/bin/terminal-assistantd
cp fish/terminal-assistant.fish ~/.config/fish/conf.d/
cp config/config.toml ~/.config/terminal-assistant/config.toml
```

---

## Quick Usage

### In the Fish Terminal
1. Type your intent in natural language:
   ```fish
   install chrome
   ```
2. Press <kbd>Ctrl+Space</kbd>. The prompt transforms instantly:
   ```fish
   paru -S google-chrome
   ```
3. Press <kbd>Enter</kbd> to execute.

### From the CLI
```bash
# Get command suggestion
terminal-assistant suggest "restart audio"

# Raw output for scripting
terminal-assistant suggest --raw "check nvidia"

# Explain a command
terminal-assistant explain "sudo pacman -Syu"

# Check daemon health and active models
terminal-assistant status
terminal-assistant models
```

---

## Example Interactions

| You Type | Assistant Suggests | Source |
|---|---|---|
| `instal chrome` | `paru -S google-chrome` | Deterministic (AUR) |
| `install discord` | `sudo pacman -S discord` | Deterministic (Arch) |
| `check nvidia` | `nvidia-smi` | Deterministic (GPU) |
| `restart audio` | `systemctl --user restart pipewire pipewire-pulse wireplumber` | Deterministic (PipeWire) |
| `what is using port 3000` | `ss -ltnp \| grep ':3000'` | Deterministic (Network) |
| `show disks` | `lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS` | Deterministic (Storage) |
| `extract archive` *(if file.zip in cwd)* | `unzip "file.zip"` | Path-Aware |
| `run project` *(if package.json in cwd)* | `npm run dev` | Path-Aware |
| `save changes` *(inside git repo)* | `git add .` | Git-Aware |
| `undo last commit` | `git reset --soft HEAD~1` | Git-Aware |
| `wipe entire nvme drive` | `sudo wipefs -a /dev/nvme0n1` | **⚠ Potentially destructive** |

---

## Configuration

Settings are stored at `~/.config/terminal-assistant/config.toml`:

```toml
[general]
shell = "fish"
ai_enabled = true
max_suggestions = 3
debounce_ms = 300
deterministic_first = true

[ai]
provider = "ollama"
model = "auto"
endpoint = "http://127.0.0.1:11434"
timeout_ms = 2500

[ui]
ghost_text = true
popup = false

[keybindings]
accept = "right"
show = "ctrl-space"
dismiss = "escape"

[safety]
warn_destructive = true
never_auto_execute = true
```

---

## Managing the Background Daemon

```bash
# Start user daemon via systemd
systemctl --user start terminal-assistant.service

# Enable auto-start on user session login
systemctl --user enable terminal-assistant.service

# View daemon logs
journalctl --user -u terminal-assistant.service -f
```

---

## Testing

Run the test suite:
```bash
cargo test
```
