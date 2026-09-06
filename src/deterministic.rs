use crate::context::SystemContext;
use crate::fuzzy::{fuzzy_find_best, matches_fuzzy};
use crate::protocol::{CandidateSuggestion, RiskLevel};
use regex::Regex;

pub fn match_deterministic(input: &str, ctx: &SystemContext) -> Vec<CandidateSuggestion> {
    let mut suggestions = Vec::new();
    let lower = input.trim().to_lowercase();
    if lower.is_empty() {
        return suggestions;
    }

    // 1. Shell Typos & Quick Navigation
    if let Some(sugs) = match_shell_typos_and_nav(&lower) {
        return sugs;
    }

    // 2. Standard Command Flags & Smart Parameter Completion
    if let Some(sugs) = match_command_tool_flags(&lower) {
        return sugs;
    }

    // 3. Path-Aware Archive Extraction
    if lower.starts_with("extract") || lower.starts_with("unzip") || lower.starts_with("untar") || lower.starts_with("decompress") {
        if let Some(sug) = match_archive_extraction(&lower, ctx) {
            suggestions.push(sug);
            return suggestions;
        }
    }

    // 4. Path-Aware Project Execution & Building
    if let Some(sugs) = match_path_aware_projects(&lower, ctx) {
        return sugs;
    }

    // 5. Git Context Operations
    if ctx.is_git_repo {
        if let Some(sugs) = match_git_workflow(&lower, ctx) {
            return sugs;
        }
    }

    // 6. CachyOS / Arch Package Management (pacman / paru)
    if let Some(sugs) = match_package_management(&lower) {
        return sugs;
    }

    // 7. Systemd & Service Management
    if let Some(sugs) = match_systemd_and_services(&lower) {
        return sugs;
    }

    // 8. Hardware, GPU, Audio, Battery & ASUS TUF
    if let Some(sugs) = match_hardware_and_laptop(&lower) {
        return sugs;
    }

    // 9. Network, Ports, IP & Connectivity
    if let Some(sugs) = match_network_and_ports(&lower) {
        return sugs;
    }

    // 10. Files, Storage, Disks, Search & Permissions
    if let Some(sugs) = match_files_and_storage(&lower, ctx) {
        return sugs;
    }

    // 11. Process Management, Performance & Memory
    if let Some(sugs) = match_processes_and_performance(&lower) {
        return sugs;
    }

    // 12. Hyprland & Wayland Desktop Controls
    if let Some(sugs) = match_hyprland_and_desktop(&lower) {
        return sugs;
    }

    // 13. Btrfs & CachyOS Administration
    if let Some(sugs) = match_btrfs_and_cachyos(&lower) {
        return sugs;
    }

    suggestions
}

// -----------------------------------------------------------------------------
// 1. Shell Typos & Quick Navigation
// -----------------------------------------------------------------------------
fn match_shell_typos_and_nav(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    match lower {
        "sl" => sugs.push(sug("ls", "List directory contents (correcting 'sl' typo)", 0.99, RiskLevel::Low, None, "Shell")),
        "cls" | "celar" => sugs.push(sug("clear", "Clear the terminal screen", 0.99, RiskLevel::Low, None, "Shell")),
        "gti" => sugs.push(sug("git", "Git version control (correcting 'gti' typo)", 0.99, RiskLevel::Low, None, "Git")),
        "dc" => sugs.push(sug("cd", "Change directory (correcting 'dc' typo)", 0.99, RiskLevel::Low, None, "Shell")),
        "cdd" | ".." => sugs.push(sug("cd ..", "Go up one directory level", 0.99, RiskLevel::Low, None, "Shell")),
        "..." => sugs.push(sug("cd ../..", "Go up two directory levels", 0.99, RiskLevel::Low, None, "Shell")),
        "back" => sugs.push(sug("cd -", "Switch back to the previous directory", 0.98, RiskLevel::Low, None, "Shell")),
        "home" => sugs.push(sug("cd ~", "Change directory to user home directory", 0.98, RiskLevel::Low, None, "Shell")),
        "root" => sugs.push(sug("sudo -i", "Open interactive root superuser shell", 0.96, RiskLevel::Medium, None, "System")),
        "open download" | "open downloads" | "cd download" | "cd downloads" => {
            sugs.push(sug("cd ~/Downloads", "Navigate to user Downloads folder", 0.99, RiskLevel::Low, None, "Shell"));
        }
        "open documents" | "cd documents" => {
            sugs.push(sug("cd ~/Documents", "Navigate to user Documents folder", 0.99, RiskLevel::Low, None, "Shell"));
        }
        "open pictures" | "cd pictures" => {
            sugs.push(sug("cd ~/Pictures", "Navigate to user Pictures folder", 0.99, RiskLevel::Low, None, "Shell"));
        }
        "open config" | "cd config" => {
            sugs.push(sug("cd ~/.config", "Navigate to user XDG configuration folder", 0.99, RiskLevel::Low, None, "Shell"));
        }
        "open projects" | "cd projects" => {
            sugs.push(sug("cd ~/Projects", "Navigate to user Projects directory", 0.99, RiskLevel::Low, None, "Shell"));
        }
        _ => return None,
    }
    Some(sugs)
}

// -----------------------------------------------------------------------------
// 2. Standard Command Flags & Smart Parameter Completion
// -----------------------------------------------------------------------------
fn match_command_tool_flags(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    match lower {
        "tar" => {
            sugs.push(sug("tar -xvzf <archive.tar.gz>", "Extract gzipped tarball archive", 0.96, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("tar -czvf archive.tar.gz <folder>", "Create compressed tar.gz archive from directory", 0.94, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("tar --zstd -xvf <archive.tar.zst>", "Extract modern Zstandard compressed tarball", 0.92, RiskLevel::Low, None, "Storage"));
        }
        "find" => {
            sugs.push(sug("find . -type f -name \"*pattern*\"", "Search recursively for files matching pattern", 0.96, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("find . -type d -name \"*pattern*\"", "Search recursively for directories matching pattern", 0.92, RiskLevel::Low, None, "Storage"));
        }
        "grep" => {
            sugs.push(sug("grep -rnw . -e \"pattern\"", "Recursively search all files for exact word match", 0.96, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("rg \"pattern\"", "Fast recursive content search using ripgrep", 0.94, RiskLevel::Low, None, "Storage"));
        }
        "rsync" => {
            sugs.push(sug("rsync -avzP source/ destination/", "Archive sync files with compression and live progress", 0.97, RiskLevel::Low, None, "Storage"));
        }
        "curl" => {
            sugs.push(sug("curl -fsSL https://", "Download URL silently following redirects", 0.95, RiskLevel::Low, None, "Network"));
            sugs.push(sug("curl -I https://", "Inspect HTTP response headers only", 0.92, RiskLevel::Low, None, "Network"));
        }
        "chmod" => {
            sugs.push(sug("chmod +x <file>", "Grant executable permission to target file", 0.97, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("chmod 644 <file>", "Set standard read/write user, read-only group permissions", 0.92, RiskLevel::Low, None, "Storage"));
        }
        "chown" => {
            sugs.push(sug("sudo chown -R $USER:$USER .", "Recursively claim ownership of current folder for current user", 0.97, RiskLevel::Medium, None, "Storage"));
        }
        "ffmpeg" => {
            sugs.push(sug("ffmpeg -i input.mp4 -c:v libx264 -crf 23 output.mp4", "Compress and re-encode video using H.264", 0.96, RiskLevel::Low, None, "Media"));
            sugs.push(sug("ffmpeg -i input.mp4 -vn -c:a libmp3lame audio.mp3", "Extract audio track from video file as MP3", 0.93, RiskLevel::Low, None, "Media"));
        }
        "ssh" => {
            sugs.push(sug("ssh user@hostname", "Establish secure interactive shell session to remote host", 0.95, RiskLevel::Low, None, "Network"));
        }
        "docker ps" => {
            sugs.push(sug("docker ps -a", "List all running, stopped, and exited Docker containers", 0.98, RiskLevel::Low, None, "Docker"));
        }
        "docker stop" => {
            sugs.push(sug("docker stop $(docker ps -q)", "Gracefully stop all running Docker containers", 0.96, RiskLevel::Medium, None, "Docker"));
        }
        _ => return None,
    }
    Some(sugs)
}

// -----------------------------------------------------------------------------
// 3. Path-Aware Project Execution & Building
// -----------------------------------------------------------------------------
fn match_path_aware_projects(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    let is_run = lower == "run project" || lower == "start project" || lower == "run app" || lower == "start dev" || lower == "dev";
    let is_build = lower == "build project" || lower == "build app" || lower == "compile";
    let is_test = lower == "test project" || lower == "run tests" || lower == "test";

    if !is_run && !is_build && !is_test {
        return None;
    }

    if ctx.files.iter().any(|f| f == "package.json") {
        let runner = if ctx.files.iter().any(|f| f == "pnpm-lock.yaml") {
            "pnpm"
        } else if ctx.files.iter().any(|f| f == "bun.lockb" || f == "bun.lock") {
            "bun"
        } else if ctx.files.iter().any(|f| f == "yarn.lock") {
            "yarn"
        } else {
            "npm"
        };

        if is_run {
            sugs.push(sug(&format!("{} run dev", runner), "Start frontend/Node.js development server", 0.99, RiskLevel::Low, None, "Project"));
            sugs.push(sug(&format!("{} start", runner), "Start production application", 0.92, RiskLevel::Low, None, "Project"));
        } else if is_build {
            sugs.push(sug(&format!("{} run build", runner), "Build frontend/Node.js bundle for production", 0.99, RiskLevel::Low, None, "Project"));
        } else if is_test {
            sugs.push(sug(&format!("{} test", runner), "Run automated project test suite", 0.99, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    if ctx.files.iter().any(|f| f == "Cargo.toml") {
        if is_run {
            sugs.push(sug("cargo run", "Compile and run Rust binary target", 0.99, RiskLevel::Low, None, "Project"));
        } else if is_build {
            sugs.push(sug("cargo build --release", "Build optimized release Rust binary", 0.99, RiskLevel::Low, None, "Project"));
        } else if is_test {
            sugs.push(sug("cargo test", "Execute all Rust unit and integration tests", 0.99, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    if ctx.files.iter().any(|f| f == "Makefile") {
        if is_run || is_build {
            sugs.push(sug("make", "Build project targets using Makefile", 0.98, RiskLevel::Low, None, "Project"));
        } else if is_test {
            sugs.push(sug("make test", "Execute tests defined in Makefile", 0.98, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    if ctx.files.iter().any(|f| f == "docker-compose.yml" || f == "compose.yaml") {
        if is_run {
            sugs.push(sug("docker compose up -d", "Start multi-container Docker services in background", 0.98, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    if ctx.files.iter().any(|f| f == "main.py") {
        if is_run {
            sugs.push(sug("python main.py", "Execute main Python entrypoint script", 0.98, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    if ctx.files.iter().any(|f| f == "go.mod") {
        if is_run {
            sugs.push(sug("go run .", "Compile and run Go package in current directory", 0.98, RiskLevel::Low, None, "Project"));
        } else if is_build {
            sugs.push(sug("go build .", "Compile Go application binary", 0.98, RiskLevel::Low, None, "Project"));
        } else if is_test {
            sugs.push(sug("go test ./...", "Run all Go tests recursively", 0.98, RiskLevel::Low, None, "Project"));
        }
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 4. Git Context Operations
// -----------------------------------------------------------------------------
fn match_git_workflow(lower: &str, _ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    if lower == "save changes" || lower == "commit changes" || lower == "git save" || matches_fuzzy(lower, "save changes", 0.8) {
        sugs.push(sug("git add .", "Stage all modified and untracked files", 0.99, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git commit -m \"Update\"", "Commit staged changes with message", 0.94, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "undo last commit" || lower == "undo commit" || lower == "revert commit" || matches_fuzzy(lower, "undo last commit", 0.8) {
        sugs.push(sug(
            "git reset --soft HEAD~1",
            "Undo last commit while preserving changes in staging area",
            0.98,
            RiskLevel::Medium,
            Some("Preserves your code unstaged. Use git reset --hard only if discarding all work.".to_string()),
            "Git",
        ));
        return Some(sugs);
    }

    if lower == "discard changes" || lower == "discard all changes" {
        sugs.push(sug(
            "git restore .",
            "Discard all unstaged working tree changes",
            0.95,
            RiskLevel::Medium,
            Some("Warning: unstaged modifications will be lost permanently".to_string()),
            "Git",
        ));
        return Some(sugs);
    }

    if lower == "git status" || lower == "check changes" || lower == "status" || lower == "check git" {
        sugs.push(sug("git status -s", "Show concise working tree status", 0.98, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git diff", "Inspect uncommitted code changes in working tree", 0.92, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "git diff" || lower == "show diff" || lower == "diff" {
        sugs.push(sug("git diff", "Show changes between working tree and index", 0.98, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git diff --staged", "Show staged changes ready to be committed", 0.95, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "push changes" || lower == "git push" || lower == "push" {
        sugs.push(sug("git push", "Push committed changes to remote repository", 0.98, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git push -u origin (git branch --show-current)", "Push and set upstream tracking branch", 0.92, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "pull changes" || lower == "git pull" || lower == "update repo" || lower == "pull" {
        sugs.push(sug("git pull", "Fetch and rebase/merge changes from remote tracking repository", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "list branches" || lower == "git branches" || lower == "check branches" {
        sugs.push(sug("git branch -a", "List all local and remote branches", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if let Some(caps) = Regex::new(r#"^(?:create branch|new branch|checkout -b)\s+([a-zA-Z0-9_\-\.\/]+)$"#).ok()?.captures(lower) {
        let branch = &caps[1];
        sugs.push(sug(&format!("git switch -c {}", branch), "Create and switch to new branch", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if let Some(caps) = Regex::new(r#"^(?:switch branch|checkout branch|switch to)\s+([a-zA-Z0-9_\-\.\/]+)$"#).ok()?.captures(lower) {
        let branch = &caps[1];
        sugs.push(sug(&format!("git switch {}", branch), "Switch to existing branch", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "git stash" || lower == "stash changes" || lower == "stash" {
        sugs.push(sug("git stash", "Temporarily shelter uncommitted changes in stash", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "pop stash" || lower == "restore stash" || lower == "git pop" {
        sugs.push(sug("git stash pop", "Re-apply and remove the most recently stashed changes", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "commit history" || lower == "git log" || lower == "view commits" {
        sugs.push(sug("git log --oneline --graph -n 15", "Display 15 most recent commits in compact branch tree", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 5. CachyOS / Arch Package Management (pacman / paru)
// -----------------------------------------------------------------------------
fn match_package_management(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // System Updates
    if lower == "update system" || lower == "upgrade system" || lower == "system update" || lower == "update" || lower == "upgrade" || lower == "pacman syu" {
        sugs.push(sug("sudo pacman -Syu", "Synchronize repositories and upgrade all system packages", 0.99, RiskLevel::Low, None, "Pacman"));
        sugs.push(sug("paru -Syu", "Upgrade both official and AUR packages seamlessly", 0.96, RiskLevel::Low, None, "AUR"));
        return Some(sugs);
    }

    // Mirror updates
    if lower == "update mirrors" || lower == "fastest mirrors" || lower == "rank mirrors" {
        sugs.push(sug("sudo cachyos-rate-mirrors", "Benchmark and update CachyOS and Arch package mirrors", 0.98, RiskLevel::Low, None, "Pacman"));
        sugs.push(sug("rate-mirrors arch | sudo tee /etc/pacman.d/mirrorlist", "Rate Arch Linux mirrors by download latency", 0.92, RiskLevel::Low, None, "Pacman"));
        return Some(sugs);
    }

    // Clean cache / orphans
    if lower == "clean cache" || lower == "clean pacman" || lower == "clear pacman cache" {
        sugs.push(sug("sudo pacman -Sc", "Remove old package tarballs from pacman cache", 0.97, RiskLevel::Low, None, "Pacman"));
        sugs.push(sug("paru -Scd", "Clean unused AUR and pacman cached build files", 0.94, RiskLevel::Low, None, "AUR"));
        return Some(sugs);
    }

    if lower == "clean orphans" || lower == "remove orphans" || lower == "delete orphans" {
        sugs.push(sug("sudo pacman -Rns (pacman -Qtdq)", "Find and recursively delete all orphaned packages", 0.98, RiskLevel::Destructive, Some("⚠ Removes unneeded dependencies. Verify package list before confirming.".to_string()), "Pacman"));
        return Some(sugs);
    }

    if lower == "list installed packages" || lower == "all packages" || lower == "list packages" {
        sugs.push(sug("pacman -Qe", "List all explicitly installed packages", 0.98, RiskLevel::Low, None, "Pacman"));
        return Some(sugs);
    }

    // Install intent
    if let Some(pkg) = parse_install_intent(lower) {
        let (cmd, desc, cat) = resolve_package_command(&pkg);
        sugs.push(sug(&cmd, &desc, 0.98, RiskLevel::Low, None, &cat));
        return Some(sugs);
    }

    // Find / Search package intent
    if let Some(pkg) = parse_find_package_intent(lower) {
        sugs.push(sug(&format!("paru -Ss {}", pkg), &format!("Search official Arch repositories and AUR for '{}'", pkg), 0.98, RiskLevel::Low, None, "AUR"));
        sugs.push(sug(&format!("pacman -Ss {}", pkg), &format!("Search official CachyOS/Arch repositories for '{}'", pkg), 0.92, RiskLevel::Low, None, "Pacman"));
        return Some(sugs);
    }

    // Uninstall / Remove intent
    if let Some(caps) = Regex::new(r#"^(?:uninstall|remove|delete)\s+(?:package\s+)?([a-zA-Z0-9_\-\.\+]+)$"#).ok()?.captures(lower) {
        let pkg = &caps[1];
        sugs.push(sug(
            &format!("sudo pacman -Rns {}", pkg),
            &format!("Remove package '{}' and its unused dependencies", pkg),
            0.98,
            RiskLevel::Destructive,
            Some("⚠ Removes package and related dependencies. Review list before confirming.".to_string()),
            "Pacman",
        ));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 6. Systemd & Service Management
// -----------------------------------------------------------------------------
fn match_systemd_and_services(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // Restart audio
    if lower == "restart audio" || lower == "fix audio" || lower == "reload audio" || lower == "pipewire restart" || matches_fuzzy(lower, "restart audio", 0.78) {
        sugs.push(sug("systemctl --user restart pipewire pipewire-pulse wireplumber", "Restart PipeWire audio server and WirePlumber session manager", 0.99, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    // Check audio
    if lower == "check audio" || lower == "audio status" || lower == "sound status" {
        sugs.push(sug("wpctl status", "Show active PipeWire audio sinks, sources, and volume levels", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    // Bluetooth
    if lower == "restart bluetooth" || matches_fuzzy(lower, "restart bluetooth", 0.8) {
        sugs.push(sug("sudo systemctl restart bluetooth", "Restart the Bluetooth hardware daemon", 0.98, RiskLevel::Medium, None, "Systemd"));
        return Some(sugs);
    }
    if lower == "check bluetooth" || lower == "bluetooth status" || matches_fuzzy(lower, "status bluetooth", 0.8) {
        sugs.push(sug("systemctl status bluetooth", "Inspect Bluetooth service operational status", 0.98, RiskLevel::Low, None, "Systemd"));
        return Some(sugs);
    }

    // Failed services
    if lower == "failed services" || lower == "check failed" || lower == "systemctl failed" {
        sugs.push(sug("systemctl --failed", "List all systemd system services that failed to start", 0.99, RiskLevel::Low, None, "Systemd"));
        sugs.push(sug("systemctl --user --failed", "List failed user session services", 0.94, RiskLevel::Low, None, "Systemd"));
        return Some(sugs);
    }

    // System logs
    if lower == "system logs" || lower == "check logs" || lower == "error logs" || lower == "journal" {
        sugs.push(sug("journalctl -xe", "Inspect latest system log entries with explanatory error catalog", 0.98, RiskLevel::Low, None, "Systemd"));
        return Some(sugs);
    }
    if lower == "boot logs" || lower == "check boot" {
        sugs.push(sug("journalctl -b", "View journal logs recorded during the current system boot", 0.98, RiskLevel::Low, None, "Systemd"));
        return Some(sugs);
    }
    if lower == "kernel logs" || lower == "dmesg" {
        sugs.push(sug("sudo dmesg -T | tail -n 50", "Inspect 50 most recent human-readable Linux kernel ring buffer logs", 0.98, RiskLevel::Low, None, "Systemd"));
        return Some(sugs);
    }

    // Pattern: restart/start/stop/status <service>
    let service_regex = Regex::new(r#"^(start|restart|stop|status|enable|disable)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?;
    if let Some(caps) = service_regex.captures(lower) {
        let action = &caps[1];
        let service = &caps[2];
        let is_user = service == "pipewire" || service == "wireplumber" || service == "pipewire-pulse";
        let (cmd, risk) = match action {
            "status" => (
                if is_user { format!("systemctl --user status {}", service) } else { format!("systemctl status {}", service) },
                RiskLevel::Low,
            ),
            "restart" => (
                if is_user { format!("systemctl --user restart {}", service) } else { format!("sudo systemctl restart {}", service) },
                RiskLevel::Medium,
            ),
            "start" => (
                if is_user { format!("systemctl --user start {}", service) } else { format!("sudo systemctl start {}", service) },
                RiskLevel::Medium,
            ),
            "stop" => (
                if is_user { format!("systemctl --user stop {}", service) } else { format!("sudo systemctl stop {}", service) },
                RiskLevel::Medium,
            ),
            "enable" => (
                format!("sudo systemctl enable --now {}", service),
                RiskLevel::Medium,
            ),
            "disable" => (
                format!("sudo systemctl disable {}", service),
                RiskLevel::Destructive,
            ),
            _ => return None,
        };

        sugs.push(sug(&cmd, &format!("Execute systemd '{}' on service '{}'", action, service), 0.97, risk, None, "Systemd"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 7. Hardware, GPU, Audio, Battery & ASUS TUF
// -----------------------------------------------------------------------------
fn match_hardware_and_laptop(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // NVIDIA GPU
    if lower == "check nvidia" || lower == "check gpu" || lower == "nvidia" || lower == "gpu info" || lower == "gpu temp" || matches_fuzzy(lower, "check nvidia", 0.78) {
        sugs.push(sug("nvidia-smi", "Display NVIDIA GPU utilization, temperature, and VRAM allocation", 0.99, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("watch -n 1 nvidia-smi", "Monitor NVIDIA GPU stats live every second", 0.94, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("lspci -nnk | grep -A4 -E \"VGA|3D|Display\"", "Inspect graphics hardware and active kernel drivers", 0.90, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // Hybrid GPU switching (ASUS / supergfxctl)
    if lower == "gpu mode" || lower == "switch gpu" || lower == "asus gpu" {
        sugs.push(sug("supergfxctl -g", "Query current ASUS hybrid graphics mode", 0.98, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("supergfxctl -m Dedicated", "Switch to dedicated NVIDIA RTX GPU only", 0.92, RiskLevel::Medium, None, "Hardware"));
        sugs.push(sug("supergfxctl -m Hybrid", "Switch to dynamic Intel + NVIDIA Hybrid mode", 0.92, RiskLevel::Medium, None, "Hardware"));
        return Some(sugs);
    }

    // ASUS ROG / TUF controls
    if lower == "check battery" || lower == "battery status" || lower == "battery" {
        sugs.push(sug("upower -i /org/freedesktop/UPower/devices/battery_BAT0", "Show detailed battery health, percentage, and discharge rate", 0.98, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    if lower == "battery limit" || lower == "asus charge" {
        sugs.push(sug("asusctl -c 80", "Set battery charge limit to 80% to preserve battery lifespan", 0.98, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    if lower == "asus fan" || lower == "fan mode" || lower == "power profile" || lower == "asus profile" {
        sugs.push(sug("asusctl profile -p", "Check or cycle ASUS TUF performance/fan profile", 0.98, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("asusctl profile -n", "Cycle to next ASUS power profile (Quiet / Balanced / Performance)", 0.95, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // Sensors & Temps
    if lower == "check sensors" || lower == "cpu temp" || lower == "temperatures" || lower == "sensors" {
        sugs.push(sug("sensors", "Display motherboard, CPU, and GPU temperature sensors", 0.98, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("watch sensors", "Monitor hardware thermal readings continuously", 0.92, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // Audio volume & mute
    if lower == "mute mic" || lower == "mute microphone" {
        sugs.push(sug("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle", "Toggle microphone mute on default audio source", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }
    if lower == "mute audio" || lower == "mute sound" || lower == "mute volume" {
        sugs.push(sug("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle", "Toggle audio playback mute on default output sink", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 8. Network, Ports, IP & Connectivity
// -----------------------------------------------------------------------------
fn match_network_and_ports(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // Port lookup
    if let Some(port) = parse_port_intent(lower) {
        sugs.push(sug(&format!("ss -ltnp | grep ':{}'", port), &format!("Find listening process bound to TCP port {}", port), 0.98, RiskLevel::Low, None, "Network"));
        sugs.push(sug(&format!("fuser {}/tcp", port), &format!("List process ID occupying TCP port {}", port), 0.92, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    if lower == "open ports" || lower == "listening ports" || lower == "check ports" {
        sugs.push(sug("ss -tulwn", "List all listening TCP and UDP sockets and port numbers", 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    // IP addresses
    if lower == "my ip" || lower == "check ip" || lower == "local ip" || lower == "ip address" {
        sugs.push(sug("ip -br a", "Show brief list of network interfaces and assigned local IP addresses", 0.99, RiskLevel::Low, None, "Network"));
        sugs.push(sug("curl -s ifconfig.me", "Query and display current public WAN IP address", 0.95, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }
    if lower == "public ip" || lower == "external ip" {
        sugs.push(sug("curl -s ifconfig.me", "Retrieve public external IP address via curl", 0.99, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    // Wi-Fi
    if lower == "list wifi" || lower == "scan wifi" || lower == "wifi networks" || lower == "available wifi" {
        sugs.push(sug("nmcli dev wifi list", "Scan and display nearby Wi-Fi access points and signal strength", 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }
    if lower == "wifi status" || lower == "check wifi" {
        sugs.push(sug("nmcli radio wifi", "Check whether Wi-Fi radio is enabled or disabled", 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }
    if let Some(caps) = Regex::new(r#"^(?:connect wifi|wifi connect)\s+["']?([^"']+)["']?$"#).ok()?.captures(lower) {
        let ssid = &caps[1];
        sugs.push(sug(&format!("nmcli dev wifi connect \"{}\" --ask", ssid), &format!("Connect to Wi-Fi network '{}' with interactive password prompt", ssid), 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    // Connectivity test
    if lower == "test internet" || lower == "ping test" || lower == "check internet" {
        sugs.push(sug("ping -c 4 1.1.1.1", "Send 4 ICMP ping packets to Cloudflare DNS to test connection latency", 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    if lower == "flush dns" {
        sugs.push(sug("resolvectl flush-caches", "Flush local systemd-resolved DNS resolver caches", 0.98, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 9. Files, Storage, Disks, Search & Permissions
// -----------------------------------------------------------------------------
fn match_files_and_storage(lower: &str, _ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // Disks & mounts
    if lower == "show disks" || lower == "check disks" || lower == "disk space" || lower == "list disks" || lower == "lsblk" || matches_fuzzy(lower, "show disks", 0.8) {
        sugs.push(sug("lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS", "List block devices, partition layout, and mount paths", 0.99, RiskLevel::Low, None, "Storage"));
        sugs.push(sug("df -h", "Show filesystem disk usage in human-readable gigabytes", 0.93, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    if lower == "disk usage" || lower == "free space" {
        sugs.push(sug("df -h", "Show free space of mounted filesystems in human-readable units", 0.98, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // Largest files
    if lower == "folder size" || lower == "largest files" || lower == "disk hogs" || lower == "biggest files" {
        sugs.push(sug("du -sh * | sort -hr | head -n 10", "Find the top 10 largest folders and files in current directory", 0.98, RiskLevel::Low, None, "Storage"));
        sugs.push(sug("ncdu", "Open interactive NCurses disk usage analyzer", 0.94, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // File searching
    if let Some(caps) = Regex::new(r#"^(?:find file|search file|find)\s+([a-zA-Z0-9_\-\.\*\?]+)$"#).ok()?.captures(lower) {
        let name = &caps[1];
        sugs.push(sug(&format!("fd \"{}\"", name), &format!("Fast search for files matching '{}' using fd", name), 0.98, RiskLevel::Low, None, "Storage"));
        sugs.push(sug(&format!("find . -name \"*{}*\"", name), &format!("Search for files containing '{}' using standard find", name), 0.92, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // Text grep
    if let Some(caps) = Regex::new(r#"^(?:search text|find text|grep)\s+["']?([^"']+)["']?$"#).ok()?.captures(lower) {
        let text = &caps[1];
        sugs.push(sug(&format!("rg \"{}\"", text), &format!("Search file contents for string '{}' using ripgrep", text), 0.98, RiskLevel::Low, None, "Storage"));
        sugs.push(sug(&format!("grep -rnw . -e \"{}\"", text), &format!("Search recursively for '{}' using standard grep", text), 0.92, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // Permissions: make executable
    if let Some(caps) = Regex::new(r#"^(?:make executable|chmod x)\s+([a-zA-Z0-9_\-\.\/]+)$"#).ok()?.captures(lower) {
        let file = &caps[1];
        sugs.push(sug(&format!("chmod +x {}", file), &format!("Grant executable permission to '{}'", file), 0.98, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // Line counting
    if lower == "count lines" || lower == "line count" {
        sugs.push(sug("wc -l *", "Count lines of text in all files in current directory", 0.98, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }
    if lower == "count files" {
        sugs.push(sug("ls -1 | wc -l", "Count total number of items in current directory", 0.98, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    // Archive creation
    if let Some(caps) = Regex::new(r#"^(?:create zip|make zip|zip folder)\s+([a-zA-Z0-9_\-\.]+)(?:\s+([a-zA-Z0-9_\-\.]+))?$"#).ok()?.captures(lower) {
        let zip_name = &caps[1];
        let target = caps.get(2).map(|m| m.as_str()).unwrap_or(".");
        let final_zip = if zip_name.ends_with(".zip") { zip_name.to_string() } else { format!("{}.zip", zip_name) };
        sugs.push(sug(&format!("zip -r \"{}\" {}", final_zip, target), &format!("Compress '{}' into zip archive '{}'", target, final_zip), 0.98, RiskLevel::Low, None, "Storage"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 10. Process Management, Performance & Memory
// -----------------------------------------------------------------------------
fn match_processes_and_performance(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // RAM
    if lower == "check ram" || lower == "check memory" || lower == "free memory" || lower == "ram usage" || lower == "free ram" || matches_fuzzy(lower, "check ram", 0.8) {
        sugs.push(sug("free -h", "Show total, allocated, and available system RAM and swap", 0.99, RiskLevel::Low, None, "Memory"));
        return Some(sugs);
    }
    if lower == "clear ram cache" || lower == "drop caches" || lower == "free ram cache" {
        sugs.push(sug("sudo sync; echo 3 | sudo tee /proc/sys/vm/drop_caches", "Flush memory pages and clean Linux kernel inode/dentry caches", 0.98, RiskLevel::Medium, None, "Memory"));
        return Some(sugs);
    }

    // Processes & Activity Monitor
    if lower == "top processes" || lower == "check cpu" || lower == "task manager" || lower == "system monitor" || lower == "btop" {
        sugs.push(sug("btop", "Open btop interactive resource, CPU, and process monitor", 0.99, RiskLevel::Low, None, "Processes"));
        sugs.push(sug("htop", "Launch htop process viewer", 0.94, RiskLevel::Low, None, "Processes"));
        return Some(sugs);
    }

    // Kill process
    if let Some(caps) = Regex::new(r#"^(?:kill process|stop process|kill)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?.captures(lower) {
        let proc = &caps[1];
        sugs.push(sug(&format!("pkill -f \"{}\"", proc), &format!("Terminate all active processes matching '{}'", proc), 0.97, RiskLevel::Medium, Some("Terminates matching process".to_string()), "Processes"));
        return Some(sugs);
    }

    if let Some(caps) = Regex::new(r#"^(?:find process|check process|pgrep)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?.captures(lower) {
        let proc = &caps[1];
        sugs.push(sug(&format!("pgrep -fl \"{}\"", proc), &format!("List process IDs and command lines matching '{}'", proc), 0.98, RiskLevel::Low, None, "Processes"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 11. Hyprland & Wayland Desktop Controls
// -----------------------------------------------------------------------------
fn match_hyprland_and_desktop(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    if lower == "reload hyprland" || lower == "reload config" || lower == "hyprland reload" {
        sugs.push(sug("hyprctl reload", "Reload Hyprland compositor configuration live without exiting", 0.99, RiskLevel::Low, None, "Hyprland"));
        return Some(sugs);
    }

    if lower == "list windows" || lower == "show windows" || lower == "hyprland clients" {
        sugs.push(sug("hyprctl clients", "List all open Wayland windows, workspaces, and window classes", 0.98, RiskLevel::Low, None, "Hyprland"));
        return Some(sugs);
    }

    if lower == "active window" || lower == "current window" {
        sugs.push(sug("hyprctl activewindow", "Inspect title, PID, and address of the currently focused window", 0.98, RiskLevel::Low, None, "Hyprland"));
        return Some(sugs);
    }

    if lower == "list monitors" || lower == "check monitors" || lower == "displays" {
        sugs.push(sug("hyprctl monitors", "Display connected monitors, active resolutions, scaling, and refresh rates", 0.98, RiskLevel::Low, None, "Hyprland"));
        return Some(sugs);
    }

    if lower == "restart quickshell" || lower == "restart shell" || lower == "reload quickshell" {
        sugs.push(sug("killall quickshell; quickshell &", "Restart Quickshell / Caelestia desktop shell in background", 0.98, RiskLevel::Medium, None, "Desktop"));
        return Some(sugs);
    }

    if lower == "screenshot" || lower == "take screenshot" || lower == "screen capture" {
        sugs.push(sug("grim -g \"$(slurp)\" ~/Pictures/screenshot_(date +%Y%m%d_%H%M%S).png", "Select a screen region and save screenshot to Pictures folder", 0.98, RiskLevel::Low, None, "Desktop"));
        sugs.push(sug("hyprshot -m region", "Interactive Hyprland region screenshot tool", 0.93, RiskLevel::Low, None, "Desktop"));
        return Some(sugs);
    }

    if lower == "lock screen" || lower == "lock" {
        sugs.push(sug("hyprlock", "Lock current Wayland session using hyprlock", 0.98, RiskLevel::Low, None, "Desktop"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 12. Btrfs & CachyOS Administration
// -----------------------------------------------------------------------------
fn match_btrfs_and_cachyos(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    if lower == "btrfs space" || lower == "btrfs df" || lower == "btrfs status" {
        sugs.push(sug("btrfs filesystem df /", "Show detailed Btrfs space allocation for root filesystem", 0.99, RiskLevel::Low, None, "Btrfs"));
        sugs.push(sug("sudo btrfs filesystem usage /", "Display full device allocation and unallocated space on root Btrfs volume", 0.95, RiskLevel::Low, None, "Btrfs"));
        return Some(sugs);
    }

    if lower == "btrfs snapshots" || lower == "list snapshots" || lower == "snapshots" {
        sugs.push(sug("sudo snapper list", "List all Snapper Btrfs system recovery snapshots", 0.98, RiskLevel::Low, None, "Btrfs"));
        sugs.push(sug("sudo btrfs subvolume list /", "List all Btrfs subvolumes in the root hierarchy", 0.92, RiskLevel::Low, None, "Btrfs"));
        return Some(sugs);
    }

    if lower == "btrfs scrub" || lower == "scrub btrfs" {
        sugs.push(sug("sudo btrfs scrub start /", "Start background integrity verification of all Btrfs data and metadata", 0.98, RiskLevel::Medium, None, "Btrfs"));
        return Some(sugs);
    }

    if lower == "kernel info" || lower == "cachyos kernel" || lower == "check kernel" {
        sugs.push(sug("uname -r", "Display running Linux kernel release and CachyOS architecture variant", 0.98, RiskLevel::Low, None, "System"));
        sugs.push(sug("cachyos-kernel-manager", "Open CachyOS graphical kernel manager utility", 0.92, RiskLevel::Low, None, "System"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// Helper parsers & lookups
// -----------------------------------------------------------------------------
fn parse_install_intent(lower: &str) -> Option<String> {
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let first = tokens[0];
    let is_install_action = first == "install"
        || first == "instal"
        || first == "isntall"
        || first == "instll"
        || first == "intall"
        || first == "get"
        || first == "add"
        || first == "download"
        || first == "fetch"
        || matches_fuzzy(first, "install", 0.78);

    if !is_install_action || tokens.len() < 2 {
        return None;
    }

    let mut pkg_tokens = &tokens[1..];
    if pkg_tokens.first() == Some(&"aur") && pkg_tokens.len() > 1 {
        pkg_tokens = &pkg_tokens[1..];
    }
    if pkg_tokens.first() == Some(&"package") && pkg_tokens.len() > 1 {
        pkg_tokens = &pkg_tokens[1..];
    }

    Some(pkg_tokens.join(" "))
}

fn parse_find_package_intent(lower: &str) -> Option<String> {
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let first = tokens[0];
    let is_find_action = first == "find"
        || first == "search"
        || first == "lookup"
        || first == "locate"
        || matches_fuzzy(first, "search", 0.78);

    if !is_find_action || tokens.len() < 2 {
        return None;
    }

    let mut pkg_tokens = &tokens[1..];
    if pkg_tokens.first() == Some(&"package") && pkg_tokens.len() > 1 {
        pkg_tokens = &pkg_tokens[1..];
    }

    Some(pkg_tokens.join(" "))
}

const POPULAR_PACKAGES: &[(&str, &str, &str, &str)] = &[
    // (fuzzy_key, binary_pkg, description, category)
    ("chrome", "google-chrome", "Google Chrome web browser", "AUR"),
    ("google chrome", "google-chrome", "Google Chrome web browser", "AUR"),
    ("spotify", "spotify", "Spotify music streaming client", "AUR"),
    ("brave", "brave-bin", "Brave privacy web browser", "AUR"),
    ("wezterm", "wezterm-git", "WezTerm GPU-accelerated terminal emulator", "AUR"),
    ("visual studio code", "visual-studio-code-bin", "Visual Studio Code proprietary binary", "AUR"),
    ("vscode", "code", "Visual Studio Code (OSS build)", "Pacman"),
    ("code", "code", "Visual Studio Code (OSS build)", "Pacman"),
    ("discord", "discord", "Discord voice and text communication client", "Pacman"),
    ("steam", "steam", "Valve Steam gaming platform", "Pacman"),
    ("lutris", "lutris", "Open gaming platform for Linux", "Pacman"),
    ("obs", "obs-studio", "OBS Studio video recording and live streaming suite", "Pacman"),
    ("vlc", "vlc", "VLC multimedia player and framework", "Pacman"),
    ("gimp", "gimp", "GNU Image Manipulation Program", "Pacman"),
    ("blender", "blender", "Blender 3D computer graphics suite", "Pacman"),
    ("telegram", "telegram-desktop", "Telegram desktop messaging client", "Pacman"),
    ("signal", "signal-desktop", "Signal private messenger desktop", "Pacman"),
    ("neovim", "neovim", "Vim-fork focused on extensibility and usability", "Pacman"),
    ("nvim", "neovim", "Vim-fork text editor", "Pacman"),
    ("emacs", "emacs", "Extensible, customizable GNU text editor", "Pacman"),
    ("firefox", "firefox", "Mozilla Firefox web browser", "Pacman"),
    ("fastfetch", "fastfetch", "Neofetch-like tool for fetching system information", "Pacman"),
    ("btop", "btop", "Resource monitor that shows usage and stats", "Pacman"),
    ("docker", "docker docker-compose", "Docker containerization engine and compose", "Pacman"),
    ("alacritty", "alacritty", "Cross-platform, GPU-accelerated terminal emulator", "Pacman"),
    ("kitty", "kitty", "Modern, hackable, GPU-based terminal emulator", "Pacman"),
    ("foot", "foot", "Fast, lightweight Wayland terminal emulator", "Pacman"),
    ("mpv", "mpv", "Lightweight command-line video player", "Pacman"),
    ("ffmpeg", "ffmpeg", "Complete, cross-platform solution to record and convert media", "Pacman"),
    ("ripgrep", "ripgrep", "Line-oriented search tool that recursively searches current directory", "Pacman"),
    ("fd", "fd", "Simple, fast and user-friendly alternative to find", "Pacman"),
    ("bat", "bat", "A cat clone with syntax highlighting and Git integration", "Pacman"),
    ("eza", "eza", "A modern, maintained replacement for ls", "Pacman"),
    ("zsh", "zsh", "Z shell command interpreter", "Pacman"),
    ("tmux", "tmux", "Terminal multiplexer", "Pacman"),
    ("htop", "htop", "Interactive process viewer", "Pacman"),
];

fn resolve_package_command(pkg: &str) -> (String, String, String) {
    let lower_pkg = pkg.to_lowercase();

    // 1. Exact match
    for &(key, target_pkg, desc, cat) in POPULAR_PACKAGES {
        if lower_pkg == key || lower_pkg == target_pkg {
            let cmd = if cat == "AUR" {
                format!("paru -S {}", target_pkg)
            } else {
                format!("sudo pacman -S {}", target_pkg)
            };
            return (cmd, format!("Install {} ({})", desc, cat), cat.to_string());
        }
    }

    // 2. Fuzzy match
    let keys: Vec<&str> = POPULAR_PACKAGES.iter().map(|p| p.0).collect();
    if let Some((best_key, score)) = fuzzy_find_best(&lower_pkg, &keys, 0.72) {
        if let Some(&(_, target_pkg, desc, cat)) = POPULAR_PACKAGES.iter().find(|p| p.0 == best_key) {
            let cmd = if cat == "AUR" {
                format!("paru -S {}", target_pkg)
            } else {
                format!("sudo pacman -S {}", target_pkg)
            };
            return (cmd, format!("Install {} (fuzzy match {:.0}%)", desc, score * 100.0), cat.to_string());
        }
    }

    // 3. Fallback
    (
        format!("sudo pacman -S {}", pkg),
        format!("Install '{}' via pacman (or paru if AUR)", pkg),
        "Pacman".to_string(),
    )
}

fn parse_port_intent(lower: &str) -> Option<String> {
    if !lower.contains("port") && !lower.contains("prt") && !lower.contains("listen") {
        return None;
    }
    let re = Regex::new(r#"\b(\d{1,5})\b"#).ok()?;
    let caps = re.captures(lower)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn match_archive_extraction(lower: &str, ctx: &SystemContext) -> Option<CandidateSuggestion> {
    // Check if user specified a hint or partial filename
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    let query_hint = if tokens.len() > 1 { Some(tokens[1]) } else { None };

    let mut matched_file = None;

    if let Some(hint) = query_hint {
        if hint != "archive" && hint != "zip" && hint != "tar" {
            for f in &ctx.files {
                let fl = f.to_lowercase();
                if fl.contains(hint) || matches_fuzzy(&fl, hint, 0.65) {
                    matched_file = Some(f.clone());
                    break;
                }
            }
        }
    }

    if matched_file.is_none() {
        matched_file = ctx.files.iter().find(|f| {
            let fl = f.to_lowercase();
            fl.ends_with(".zip")
                || fl.ends_with(".tar.gz")
                || fl.ends_with(".tgz")
                || fl.ends_with(".tar.xz")
                || fl.ends_with(".tar.zst")
                || fl.ends_with(".7z")
                || fl.ends_with(".tar.bz2")
        }).cloned();
    }

    if let Some(archive) = matched_file {
        let (cmd, tool) = if archive.ends_with(".zip") {
            (format!("unzip \"{}\"", archive), "unzip")
        } else if archive.ends_with(".tar.gz") || archive.ends_with(".tgz") {
            (format!("tar -xvzf \"{}\"", archive), "tar")
        } else if archive.ends_with(".tar.xz") {
            (format!("tar -xvJf \"{}\"", archive), "tar")
        } else if archive.ends_with(".tar.zst") {
            (format!("tar --zstd -xvf \"{}\"", archive), "tar")
        } else if archive.ends_with(".tar.bz2") {
            (format!("tar -xvjf \"{}\"", archive), "tar")
        } else {
            (format!("7z x \"{}\"", archive), "7z")
        };

        Some(sug(&cmd, &format!("Extract archive '{}' using {}", archive, tool), 0.98, RiskLevel::Low, None, "Storage"))
    } else {
        Some(sug("unzip <file>.zip", "Extract zip archive to current directory", 0.88, RiskLevel::Low, None, "Storage"))
    }
}

fn sug(command: &str, description: &str, confidence: f32, risk: RiskLevel, warning: Option<String>, category: &str) -> CandidateSuggestion {
    CandidateSuggestion {
        command: command.to_string(),
        description: description.to_string(),
        confidence,
        risk,
        warning,
        source: "deterministic".to_string(),
        category: Some(category.to_string()),
    }
}
