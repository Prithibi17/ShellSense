# Contributing to ShellSense

Thank you for your interest in contributing to **ShellSense**! 🚀

ShellSense is engineered to be an ultra-fast, sub-millisecond, offline-first Linux command assistant and autocomplete layer that works across all Linux distributions and hardware configurations.

---

## Core Architecture Principles

1. **Sub-millisecond Latency (< 1ms)**:
   - System detection relies strictly on zero-overhead Linux kernel APIs and sysfs (`/sys/class/drm`, `/sys/class/power_supply/`, `/etc/os-release`).
   - Never spawn blocking shell processes during completions.
2. **Keyboard Safety Guarantee**:
   - ShellSense **must never** intercept or lag normal printable keystrokes.
   - Completions hook exclusively into standard completion keys (<kbd>Tab</kbd>, <kbd>→</kbd>).
3. **Universal Distro & Hardware Portability**:
   - Any rule or feature should support the diverse Linux ecosystem: Arch, Debian, Ubuntu, Fedora, openSUSE, Alpine, Void, etc.
   - Hardware detection must adapt gracefully across NVIDIA, AMD Radeon, Intel Arc/iGPU, and headless servers.

---

## Development Setup

### Prerequisites
- Rust 1.80+ (`rustup default stable`)
- Git
- Fish, Bash, or Zsh for testing shell integration

### Clone and Build
```bash
git clone https://github.com/Prithibi17/ShellSense.git
cd shellsense
cargo build
```

### Running Tests
Run the full test suite with:
```bash
cargo test
```

---

## Adding New Commands & Knowledge

### 1. Adding a Software Package Mapping
Open [`src/deterministic.rs`](src/deterministic.rs). In the `PACKAGE_ALIASES` map or `try_package_install` function, add the new software name and its package targets across distributions:

```rust
("obs", "obs-studio"),
("docker", "docker"),
```

### 2. Adding Hardware-Adaptive Actions
Add intent patterns in `src/deterministic.rs` inside `evaluate_deterministic(...)`:

```rust
// Example: audio routing or control
if input_lower.contains("mute mic") {
    return Some(context.audio.mute_microphone_command());
}
```

### 3. Adding New Distro / Hardware Detection
Modify [`src/context.rs`](src/context.rs). Ensure all detection uses direct file reads or sysfs queries without spawning subshells.

---

## Submitting a Pull Request

1. Fork the repository and create a feature branch:
   ```bash
   git checkout -b feat/my-new-feature
   ```
2. Write unit tests in `src/deterministic.rs` or `tests/integration_tests.rs` for your changes.
3. Ensure all tests pass:
   ```bash
   cargo test
   ```
4. Format code:
   ```bash
   cargo fmt --check
   ```
5. Open a Pull Request with a clear description of the new intent or capability!
