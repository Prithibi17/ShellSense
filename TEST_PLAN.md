# Test Plan: ShellSense (`shellsense`)

This document outlines automated and manual verification procedures for `shellsense` on any Linux distribution with Fish, Bash, or Zsh.

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
| `safety.rs` | `test_apt_correction` | `apt install vlc` converted to `sudo pacman -S vlc` on Arch-based systems |
| `safety.rs` | `test_safe_commands` | Read-only & service restarts classified as non-destructive |
| `context.rs` | `test_sanitize_secrets` | `OPENAI_API_KEY=...` replaced with `[REDACTED]` |
| `context.rs` | `test_sanitize_bearer` | `Bearer ...` token values masked |
| `deterministic.rs` | `test_install_chrome` | `instal chrome` -> `paru -S google-chrome` |
| `deterministic.rs` | `test_nvidia_gpu` | `check nvidia` -> `nvidia-smi` |
| `deterministic.rs` | `test_restart_audio` | `restart audio` -> `systemctl --user restart pipewire wireplumber` |
| `deterministic.rs` | `test_path_aware_zip` | `extract archive` with `archive.zip` in cwd -> `unzip "archive.zip"` |
| `deterministic.rs` | `test_path_aware_project` | `run project` with `package.json` -> `npm run dev` |
| `integration_tests.rs` | `test_git_heuristics` | `save changes` -> `git add .`, `undo last commit` -> `git reset --soft HEAD~1` |
| `integration_tests.rs` | `test_distro_correction_to_arch` | `apt update` -> `sudo pacman -Sy`, `dnf install` -> `sudo pacman -S` |

---

## 2. Interactive CLI Verification

### Test 2.1: Daemon Lifecycle and Status
```bash
# Check status via CLI
ss status
# Or full command
shellsense status
```
**Pass Criteria:**
- Reports `● Running` with active socket path `$XDG_RUNTIME_DIR/shellsense.sock`.
- Outputs detected Linux distro, package manager, GPU vendor, and audio server.

### Test 2.2: Deterministic Command Suggestions
```bash
ss suggest "instal chrome"
ss suggest "check gpu"
ss suggest "restart audio"
ss suggest "what is using port 3000"
```
**Pass Criteria:**
- Returns correct native shell command with sub-millisecond local latency.

---

## 3. Shell Interactive Verification

### Test 3.1: Fish Shell
1. Type `instal chrome` and hit <kbd>Tab</kbd> -> line completes to `paru -S google-chrome`.
2. Hit <kbd>Esc</kbd> -> reverts back to `instal chrome`.
3. Normal typing must remain completely fluid with zero lag or freezing.

### Test 3.2: Bash Shell
1. Type `check gpu` and hit <kbd>Tab</kbd> -> line completes to `nvidia-smi`.

### Test 3.3: Zsh Shell
1. Type `restart audio` and hit <kbd>Tab</kbd> -> line completes to audio restart command.
