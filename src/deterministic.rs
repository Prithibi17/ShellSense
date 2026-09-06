use crate::context::SystemContext;
use crate::protocol::{CandidateSuggestion, RiskLevel};
use regex::Regex;

pub fn match_deterministic(input: &str, ctx: &SystemContext) -> Vec<CandidateSuggestion> {
    let mut suggestions = Vec::new();
    let lower = input.trim().to_lowercase();
    if lower.is_empty() {
        return suggestions;
    }

    // 1. Path-Aware Archive Extraction
    if lower.starts_with("extract") || lower.starts_with("unzip") || lower.starts_with("untar") {
        if let Some(sug) = match_archive_extraction(&lower, ctx) {
            suggestions.push(sug);
        }
    }

    // 2. Path-Aware Project Execution
    if lower == "run project" || lower == "start project" || lower == "run app" || lower == "start dev" {
        if let Some(sug) = match_project_run(ctx) {
            suggestions.push(sug);
        }
    }

    // 3. Git Context Operations
    if ctx.is_git_repo {
        if lower == "save changes" || lower == "commit changes" || lower == "git save" {
            suggestions.push(CandidateSuggestion {
                command: "git add .".to_string(),
                description: "Stage all modified and new files".to_string(),
                confidence: 0.98,
                risk: RiskLevel::Low,
                warning: None,
                source: "deterministic".to_string(),
            });
            suggestions.push(CandidateSuggestion {
                command: "git commit -m \"Update\"".to_string(),
                description: "Commit staged changes with a message".to_string(),
                confidence: 0.90,
                risk: RiskLevel::Low,
                warning: None,
                source: "deterministic".to_string(),
            });
            return suggestions;
        } else if lower == "undo last commit" || lower == "undo commit" || lower == "revert commit" {
            suggestions.push(CandidateSuggestion {
                command: "git reset --soft HEAD~1".to_string(),
                description: "Undo last commit while preserving changes in staging area".to_string(),
                confidence: 0.96,
                risk: RiskLevel::Medium,
                warning: Some("Preserves your code unstaged. Use git reset --hard only if discarding all work.".to_string()),
                source: "deterministic".to_string(),
            });
            return suggestions;
        } else if lower == "check changes" || lower == "git diff" {
            suggestions.push(CandidateSuggestion {
                command: "git status -s".to_string(),
                description: "Show short git working tree status".to_string(),
                confidence: 0.95,
                risk: RiskLevel::Low,
                warning: None,
                source: "deterministic".to_string(),
            });
            return suggestions;
        } else if lower == "discard changes" {
            suggestions.push(CandidateSuggestion {
                command: "git restore .".to_string(),
                description: "Discard all unstaged working tree changes".to_string(),
                confidence: 0.92,
                risk: RiskLevel::Medium,
                warning: Some("Warning: unstaged changes will be discarded permanently".to_string()),
                source: "deterministic".to_string(),
            });
            return suggestions;
        }
    }

    // 4. CachyOS / Arch Package Management (pacman / paru)
    if let Some(pkg_query) = parse_install_intent(&lower) {
        let (cmd, desc) = resolve_package_command(&pkg_query);
        suggestions.push(CandidateSuggestion {
            command: cmd,
            description: desc,
            confidence: 0.97,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if let Some(pkg) = parse_find_package_intent(&lower) {
        suggestions.push(CandidateSuggestion {
            command: format!("paru -Ss {}", pkg),
            description: format!("Search official repositories and AUR for '{}'", pkg),
            confidence: 0.96,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "update system" || lower == "upgrade system" || lower == "system update" || lower == "update" {
        suggestions.push(CandidateSuggestion {
            command: "sudo pacman -Syu".to_string(),
            description: "Synchronize repositories and upgrade all system packages".to_string(),
            confidence: 0.99,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        suggestions.push(CandidateSuggestion {
            command: "paru -Syu".to_string(),
            description: "Upgrade both official and AUR packages".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    // 5. Hardware & Subsystems (NVIDIA, Audio, Bluetooth, ASUS, Ports, Disks)
    if lower == "check nvidia" || lower == "check gpu" || lower == "nvidia" || lower == "gpu info" {
        suggestions.push(CandidateSuggestion {
            command: "nvidia-smi".to_string(),
            description: "Display NVIDIA GPU status, temperature, and VRAM usage".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        suggestions.push(CandidateSuggestion {
            command: "lspci -nnk | grep -A4 -E \"VGA|3D|Display\"".to_string(),
            description: "Inspect graphics hardware and active kernel drivers".to_string(),
            confidence: 0.90,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "restart audio" || lower == "fix audio" || lower == "reload audio" {
        suggestions.push(CandidateSuggestion {
            command: "systemctl --user restart pipewire pipewire-pulse wireplumber".to_string(),
            description: "Restart PipeWire audio server and WirePlumber session manager".to_string(),
            confidence: 0.99,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "check audio" || lower == "audio status" {
        suggestions.push(CandidateSuggestion {
            command: "wpctl status".to_string(),
            description: "Show PipeWire endpoints, sinks, and volume levels".to_string(),
            confidence: 0.96,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "check bluetooth" || lower == "bluetooth status" {
        suggestions.push(CandidateSuggestion {
            command: "systemctl status bluetooth".to_string(),
            description: "Check Linux Bluetooth systemd service status".to_string(),
            confidence: 0.97,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "restart bluetooth" {
        suggestions.push(CandidateSuggestion {
            command: "sudo systemctl restart bluetooth".to_string(),
            description: "Restart the Bluetooth service".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Medium,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if let Some(port) = parse_port_intent(&lower) {
        suggestions.push(CandidateSuggestion {
            command: format!("ss -ltnp | grep ':{}'", port),
            description: format!("Find listening process bound to TCP port {}", port),
            confidence: 0.97,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "show disks" || lower == "check disks" || lower == "disk space" || lower == "list disks" {
        suggestions.push(CandidateSuggestion {
            command: "lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS".to_string(),
            description: "List block devices, partitions, filesystems, and mount points".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        suggestions.push(CandidateSuggestion {
            command: "df -h".to_string(),
            description: "Show disk usage of mounted filesystems in human-readable format".to_string(),
            confidence: 0.92,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "check ram" || lower == "check memory" || lower == "free memory" {
        suggestions.push(CandidateSuggestion {
            command: "free -h".to_string(),
            description: "Show total, used, and free system memory and swap".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "check battery" || lower == "asus fan" || lower == "power profile" {
        suggestions.push(CandidateSuggestion {
            command: "asusctl profile -p".to_string(),
            description: "Check or cycle ASUS ROG/TUF power profile".to_string(),
            confidence: 0.94,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "btrfs status" || lower == "btrfs space" {
        suggestions.push(CandidateSuggestion {
            command: "btrfs filesystem df /".to_string(),
            description: "Show detailed Btrfs space allocation for root volume".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    // 6. Hyprland & Desktop Navigation
    if lower == "reload hyprland" || lower == "reload config" || lower == "hyprland reload" {
        suggestions.push(CandidateSuggestion {
            command: "hyprctl reload".to_string(),
            description: "Reload Hyprland compositor configuration without restarting".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "list monitors" || lower == "check monitors" {
        suggestions.push(CandidateSuggestion {
            command: "hyprctl monitors".to_string(),
            description: "Show connected displays, resolutions, and refresh rates".to_string(),
            confidence: 0.97,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "open download" || lower == "open downloads" {
        suggestions.push(CandidateSuggestion {
            command: "cd ~/Downloads".to_string(),
            description: "Navigate to Downloads directory".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    if lower == "open documents" {
        suggestions.push(CandidateSuggestion {
            command: "cd ~/Documents".to_string(),
            description: "Navigate to Documents directory".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    // 7. Shell Completion Snippets (e.g. systemctl, pacman prefix)
    if lower.starts_with("systemctl res") {
        suggestions.push(CandidateSuggestion {
            command: "sudo systemctl restart ".to_string(),
            description: "Restart a systemd service".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Medium,
            warning: None,
            source: "deterministic".to_string(),
        });
        return suggestions;
    }

    suggestions
}

fn parse_install_intent(lower: &str) -> Option<String> {
    let re = Regex::new(r#"^(?:instal|install|get|download)\s+(?:aur\s+package\s+|aur\s+)?([a-zA-Z0-9_\-\.\+]+)$"#).ok()?;
    let caps = re.captures(lower)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn parse_find_package_intent(lower: &str) -> Option<String> {
    let re = Regex::new(r#"^(?:find|search)\s+(?:chrome|package\s+)?([a-zA-Z0-9_\-\.\+]+)(?:\s+package)?$"#).ok()?;
    if let Some(caps) = re.captures(lower) {
        return Some(caps.get(1)?.as_str().to_string());
    }
    None
}

fn resolve_package_command(pkg: &str) -> (String, String) {
    let lower_pkg = pkg.to_lowercase();
    match lower_pkg.as_str() {
        "chrome" | "google-chrome" => (
            "paru -S google-chrome".to_string(),
            "Install Google Chrome from AUR using paru".to_string(),
        ),
        "spotify" => (
            "paru -S spotify".to_string(),
            "Install Spotify client from AUR".to_string(),
        ),
        "vscode" | "code" => (
            "sudo pacman -S code".to_string(),
            "Install Visual Studio Code (OSS) via pacman".to_string(),
        ),
        "discord" => (
            "sudo pacman -S discord".to_string(),
            "Install Discord client via pacman".to_string(),
        ),
        "steam" => (
            "sudo pacman -S steam".to_string(),
            "Install Steam gaming client via pacman".to_string(),
        ),
        "neovim" | "nvim" => (
            "sudo pacman -S neovim".to_string(),
            "Install Neovim editor via pacman".to_string(),
        ),
        "package" => (
            "paru -S <package>".to_string(),
            "Install package from Arch repositories or AUR".to_string(),
        ),
        _ => (
            format!("sudo pacman -S {}", pkg),
            format!("Install '{}' via pacman (or paru if AUR)", pkg),
        ),
    }
}

fn parse_port_intent(lower: &str) -> Option<String> {
    let re = Regex::new(r#"(?:what is using port|check port|port|listen on port)\s+(\d{1,5})"#).ok()?;
    let caps = re.captures(lower)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn match_archive_extraction(_lower: &str, ctx: &SystemContext) -> Option<CandidateSuggestion> {
    // Check if user specified a filename or just "extract archive" / "extract zip"
    let target_file = ctx.files.iter().find(|f| {
        let fl = f.to_lowercase();
        fl.ends_with(".zip")
            || fl.ends_with(".tar.gz")
            || fl.ends_with(".tgz")
            || fl.ends_with(".tar.xz")
            || fl.ends_with(".tar.zst")
            || fl.ends_with(".7z")
    });

    if let Some(archive) = target_file {
        let (cmd, tool) = if archive.ends_with(".zip") {
            (format!("unzip \"{}\"", archive), "unzip")
        } else if archive.ends_with(".tar.gz") || archive.ends_with(".tgz") {
            (format!("tar -xvzf \"{}\"", archive), "tar")
        } else if archive.ends_with(".tar.xz") {
            (format!("tar -xvJf \"{}\"", archive), "tar")
        } else if archive.ends_with(".tar.zst") {
            (format!("tar --zstd -xvf \"{}\"", archive), "tar")
        } else {
            (format!("7z x \"{}\"", archive), "7z")
        };

        Some(CandidateSuggestion {
            command: cmd,
            description: format!("Extract '{}' using {}", archive, tool),
            confidence: 0.97,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else {
        // Fallback default
        Some(CandidateSuggestion {
            command: "unzip <file>.zip".to_string(),
            description: "Extract zip archive to current directory".to_string(),
            confidence: 0.85,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    }
}

fn match_project_run(ctx: &SystemContext) -> Option<CandidateSuggestion> {
    if ctx.files.iter().any(|f| f == "package.json") {
        Some(CandidateSuggestion {
            command: "npm run dev".to_string(),
            description: "Run Node.js development server from package.json".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else if ctx.files.iter().any(|f| f == "Cargo.toml") {
        Some(CandidateSuggestion {
            command: "cargo run".to_string(),
            description: "Build and run Rust binary project".to_string(),
            confidence: 0.98,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else if ctx.files.iter().any(|f| f == "Makefile") {
        Some(CandidateSuggestion {
            command: "make".to_string(),
            description: "Build project using Makefile".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else if ctx.files.iter().any(|f| f == "docker-compose.yml" || f == "compose.yaml") {
        Some(CandidateSuggestion {
            command: "docker compose up".to_string(),
            description: "Start multi-container Docker application".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else if ctx.files.iter().any(|f| f == "main.py") {
        Some(CandidateSuggestion {
            command: "python main.py".to_string(),
            description: "Execute main Python entrypoint".to_string(),
            confidence: 0.95,
            risk: RiskLevel::Low,
            warning: None,
            source: "deterministic".to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_context(files: Vec<&str>, is_git: bool) -> SystemContext {
        SystemContext {
            cwd: PathBuf::from("/test"),
            files: files.into_iter().map(String::from).collect(),
            is_git_repo: is_git,
            git_branch: if is_git { Some("main".into()) } else { None },
            git_modified_files: Vec::new(),
            os_id: "cachyos".into(),
            shell: "fish".into(),
            recent_history: Vec::new(),
        }
    }

    #[test]
    fn test_install_chrome() {
        let ctx = test_context(vec![], false);
        let sugs = match_deterministic("instal chrome", &ctx);
        assert!(!sugs.is_empty());
        assert_eq!(sugs[0].command, "paru -S google-chrome");
    }

    #[test]
    fn test_nvidia_gpu() {
        let ctx = test_context(vec![], false);
        let sugs = match_deterministic("check nvidia", &ctx);
        assert!(!sugs.is_empty());
        assert_eq!(sugs[0].command, "nvidia-smi");
    }

    #[test]
    fn test_restart_audio() {
        let ctx = test_context(vec![], false);
        let sugs = match_deterministic("restart audio", &ctx);
        assert!(!sugs.is_empty());
        assert_eq!(sugs[0].command, "systemctl --user restart pipewire pipewire-pulse wireplumber");
    }

    #[test]
    fn test_path_aware_zip() {
        let ctx = test_context(vec!["archive.zip", "notes.txt"], false);
        let sugs = match_deterministic("extract archive", &ctx);
        assert!(!sugs.is_empty());
        assert_eq!(sugs[0].command, "unzip \"archive.zip\"");
    }

    #[test]
    fn test_path_aware_project() {
        let ctx = test_context(vec!["package.json", "src"], false);
        let sugs = match_deterministic("run project", &ctx);
        assert!(!sugs.is_empty());
        assert_eq!(sugs[0].command, "npm run dev");
    }
}
