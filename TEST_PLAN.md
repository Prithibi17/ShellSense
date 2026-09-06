# Test Plan: Terminal Assistant (`terminal-assistant`)

This document outlines automated and manual verification procedures for `terminal-assistant` on CachyOS / Arch Linux with fish shell and Hyprland.

---

## 1. Automated Test Suite

Run all unit and integration tests:
```bash
cargo test
```

### Coverage Matrix
| Module | Test Name | Expected Result |
|---|---|---|
| `safety.rs` | `test_destructive_commands` | `rm -rf`, `mkfs.ext4`, `dd if=... of=/dev/...`, `pacman -Rns` classified as `Destructive` with warnings |
| `safety.rs` | `test_apt_correction` | `apt install vlc` converted to `sudo pacman -S vlc` |
| `safety.rs` | `test_safe_commands` | Read-only & service restarts classified as non-destructive |
| `context.rs` | `test_sanitize_secrets` | `OPENAI_API_KEY=...` replaced with `[REDACTED]` |
| `context.rs` | `test_sanitize_bearer` | `Bearer ...` token values masked |
| `deterministic.rs` | `test_install_chrome` | `instal chrome` -> `paru -S google-chrome` |
| `deterministic.rs` | `test_nvidia_gpu` | `check nvidia` -> `nvidia-smi` |
| `deterministic.rs` | `test_restart_audio` | `restart audio` -> `systemctl --user restart pipewire pipewire-pulse wireplumber` |
| `deterministic.rs` | `test_path_aware_zip` | `extract archive` with `archive.zip` in cwd -> `unzip "archive.zip"` |
| `deterministic.rs` | `test_path_aware_project` | `run project` with `package.json` -> `npm run dev` |
| `integration_tests.rs` | `test_git_heuristics` | `save changes` -> `git add .`, `undo last commit` -> `git reset --soft HEAD~1` |
| `integration_tests.rs` | `test_distro_correction_to_arch` | `apt update` -> `sudo pacman -Sy`, `dnf install` -> `sudo pacman -S` |

---

## 2. Interactive CLI Verification

### Test 2.1: Daemon Lifecycle and Status
```bash
# Check status when daemon is stopped
target/release/terminal-assistant status

# Start daemon
target/release/terminal-assistantd &
DAEMON_PID=$!

# Check status when daemon is running
target/release/terminal-assistant status
```
**Pass Criteria:**
- Reports `● Running` with active socket path `$XDG_RUNTIME_DIR/terminal-assistant.sock`.

### Test 2.2: Deterministic Command Suggestions
```bash
target/release/terminal-assistant suggest "install chrome"
target/release/terminal-assistant suggest "check nvidia"
target/release/terminal-assistant suggest "restart audio"
target/release/terminal-assistant suggest "what is using port 3000"
```
**Pass Criteria:**
- `instal chrome` -> `paru -S google-chrome`
- `check nvidia` -> `nvidia-smi`
- `restart audio` -> `systemctl --user restart pipewire pipewire-pulse wireplumber`
- `what is using port 3000` -> `ss -ltnp | grep ':3000'`

### Test 2.3: Raw Shell Replacement Mode
```bash
CMD=$(target/release/terminal-assistant suggest --raw "install discord")
echo "Suggested: $CMD"
```
**Pass Criteria:** Output is exactly `sudo pacman -S discord` with no trailing metadata.

### Test 2.4: Destructive Command Safety Warnings
```bash
target/release/terminal-assistant suggest "wipe entire nvme drive"
```
**Pass Criteria:**
- Command tagged with `[⚠ Potentially destructive]`
- Risk level is `destructive`.

### Test 2.5: Command Explanation
```bash
target/release/terminal-assistant explain "sudo pacman -Syu"
```
**Pass Criteria:**
- Explains that `pacman` synchronizes repositories and performs a full system upgrade.

---

## 3. Fish Shell Integration Verification

### Test 3.1: Interactive Shell Keybinding
```fish
source fish/terminal-assistant.fish
```
1. Type: `install chrome`
2. Press <kbd>Ctrl+Space</kbd>
3. Verify:
   - Line transforms to `paru -S google-chrome`
   - Command is **NOT** executed
   - Cursor is placed at the end of the line
   - Press <kbd>Enter</kbd> only if you wish to run it manually.

### Test 3.2: Path-Aware Extraction
1. Create a dummy zip: `touch test_doc.zip`
2. In fish, type: `extract archive`
3. Press <kbd>Ctrl+Space</kbd>
4. Verify: Line transforms to `unzip "test_doc.zip"`.
5. Clean up: `rm test_doc.zip`.

### Test 3.3: Graceful Daemon Fallback
1. Kill daemon: `kill $DAEMON_PID`
2. In fish, type: `check nvidia`
3. Press <kbd>Ctrl+Space</kbd>
4. Verify: In-process fallback executes immediately and returns `nvidia-smi` with zero terminal freeze.
