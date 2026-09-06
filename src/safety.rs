use crate::protocol::RiskLevel;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub normalized_command: String,
    pub risk: RiskLevel,
    pub warning: Option<String>,
    pub confidence_penalty: f32,
    pub corrected_distro: bool,
    pub binary_exists: bool,
}

static DESTRUCTIVE_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();
static MEDIUM_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_destructive_regexes() -> &'static Vec<Regex> {
    DESTRUCTIVE_REGEXES.get_or_init(|| {
        vec![
            Regex::new(r#"\brm\s+(-[a-zA-Z]*[rf][a-zA-Z]*|--recursive|--force)"#).unwrap(),
            Regex::new(r#"\bmkfs(\.[a-zA-Z0-9]+)?\b"#).unwrap(),
            Regex::new(r#"\bdd\b.*(\bof=/dev/)"#).unwrap(),
            Regex::new(r#"\b(pacman|paru)\s+(-[rR][a-zA-Z]*|--remove)"#).unwrap(),
            Regex::new(r#"\bsystemctl\s+(disable|mask)\b"#).unwrap(),
            Regex::new(r#"\b(reboot|shutdown|poweroff|halt|init\s+0)\b"#).unwrap(),
            Regex::new(r#"\b(chmod|chown)\s+(-[a-zA-Z]*R|--recursive)\s+/(etc|usr|var|sys|proc|boot|bin|sbin)?(\s|$)"#).unwrap(),
            Regex::new(r#">\s*/(sys|proc|dev)/"#).unwrap(),
            Regex::new(r#"\b(fdisk|parted|gdisk|wipefs|sgdisk|cfdisk)\b"#).unwrap(),
            Regex::new(r#"\bformat\b.*(\b/dev/)"#).unwrap(),
            Regex::new(r#"\bshred\b"#).unwrap(),
        ]
    })
}

fn get_medium_regexes() -> &'static Vec<Regex> {
    MEDIUM_REGEXES.get_or_init(|| {
        vec![
            Regex::new(r#"\bgit\s+reset\s+--hard\b"#).unwrap(),
            Regex::new(r#"\bgit\s+clean\s+(-[a-zA-Z]*f)"#).unwrap(),
            Regex::new(r#"\bkill\s+-9\b"#).unwrap(),
            Regex::new(r#"\bkillall\s+-9\b"#).unwrap(),
            Regex::new(r#"\bsystemctl\s+(stop|restart)\b"#).unwrap(),
            Regex::new(r#"\bpkill\b"#).unwrap(),
            Regex::new(r#"\btruncate\b"#).unwrap(),
        ]
    })
}

pub fn check_safety(cmd: &str) -> SafetyReport {
    let trimmed = cmd.trim();
    let (normalized, corrected_distro) = correct_distro_command(trimmed);

    let mut risk = RiskLevel::Low;
    let mut warning = None;
    let mut penalty: f32 = 0.0;

    // 1. Destructive check
    for re in get_destructive_regexes() {
        if re.is_match(&normalized) {
            risk = RiskLevel::Destructive;
            warning = Some("⚠ Potentially destructive: modifies partitions, deletes files, or alters system integrity".to_string());
            break;
        }
    }

    // 2. Medium risk check
    if risk == RiskLevel::Low {
        for re in get_medium_regexes() {
            if re.is_match(&normalized) {
                risk = RiskLevel::Medium;
                warning = Some("Notice: modifies active services or uncommitted state".to_string());
                break;
            }
        }
    }

    // 3. Binary verification in PATH
    let (binary_exists, bin_name) = verify_binary_exists(&normalized);
    if !binary_exists && !bin_name.is_empty() {
        penalty += 0.25;
    }

    SafetyReport {
        normalized_command: normalized,
        risk,
        warning,
        confidence_penalty: penalty,
        corrected_distro,
        binary_exists,
    }
}

pub fn correct_distro_command(cmd: &str) -> (String, bool) {
    let mut modified = cmd.to_string();
    let mut changed = false;

    // Replace Ubuntu/Debian 'apt install' -> 'sudo pacman -S'
    let apt_install = Regex::new(r#"(?i)\b(sudo\s+)?apt(-get)?\s+install\s+(-y\s+)?(.*)"#).unwrap();
    if let Some(caps) = apt_install.captures(&modified) {
        let pkgs = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        modified = format!("sudo pacman -S {}", pkgs.trim());
        changed = true;
    } else if Regex::new(r#"(?i)\b(sudo\s+)?apt(-get)?\s+update\b"#).unwrap().is_match(&modified) {
        modified = "sudo pacman -Sy".to_string();
        changed = true;
    } else if Regex::new(r#"(?i)\b(sudo\s+)?apt(-get)?\s+upgrade\b"#).unwrap().is_match(&modified) {
        modified = "sudo pacman -Syu".to_string();
        changed = true;
    } else if let Some(caps) = Regex::new(r#"(?i)\b(sudo\s+)?(dnf|yum)\s+install\s+(-y\s+)?(.*)"#).unwrap().captures(&modified) {
        let pkgs = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        modified = format!("sudo pacman -S {}", pkgs.trim());
        changed = true;
    }

    (modified, changed)
}

fn verify_binary_exists(cmd: &str) -> (bool, String) {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    if tokens.is_empty() {
        return (true, String::new());
    }

    // Skip sudo or env prefix
    let mut idx = 0;
    while idx < tokens.len() && (tokens[idx] == "sudo" || tokens[idx] == "env" || tokens[idx].starts_with('-')) {
        idx += 1;
    }

    if idx >= tokens.len() {
        return (true, String::new());
    }

    let bin = tokens[idx];

    // Shell builtins & control
    let builtins = [
        "cd", "echo", "pwd", "exit", "set", "source", "type", "alias", "unalias", "test",
        "export", "read", "true", "false", "bind", "commandline", "functions", "complete",
        "history", "jobs", "fg", "bg", "wait", "eval", "exec", "help",
    ];
    if builtins.contains(&bin) {
        return (true, bin.to_string());
    }

    // If it's a path like ./script.sh or /usr/bin/foo
    if bin.contains('/') {
        return (Path::new(bin).exists(), bin.to_string());
    }

    // Check PATH environment variable
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let full_path = Path::new(dir).join(bin);
            if full_path.is_file() {
                return (true, bin.to_string());
            }
        }
    }

    // Common standard paths fallback
    let common_paths = [
        "/usr/bin", "/usr/local/bin", "/bin", "/sbin", "/usr/sbin",
    ];
    for dir in &common_paths {
        if Path::new(dir).join(bin).is_file() {
            return (true, bin.to_string());
        }
    }

    // Special packages or placeholders (e.g. <package>, google-chrome when installing)
    if bin == "paru" || bin == "pacman" || bin == "git" || bin == "hyprctl" || bin == "systemctl" {
        return (true, bin.to_string());
    }

    (false, bin.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destructive_commands() {
        let rep1 = check_safety("rm -rf /home/prithibi/test");
        assert_eq!(rep1.risk, RiskLevel::Destructive);
        assert!(rep1.warning.is_some());

        let rep2 = check_safety("sudo mkfs.ext4 /dev/nvme0n1p3");
        assert_eq!(rep2.risk, RiskLevel::Destructive);

        let rep3 = check_safety("sudo dd if=/dev/zero of=/dev/sda bs=1M");
        assert_eq!(rep3.risk, RiskLevel::Destructive);

        let rep4 = check_safety("sudo pacman -Rns discord");
        assert_eq!(rep4.risk, RiskLevel::Destructive);
    }

    #[test]
    fn test_apt_correction() {
        let (corrected, changed) = correct_distro_command("apt install vlc");
        assert!(changed);
        assert_eq!(corrected, "sudo pacman -S vlc");
    }

    #[test]
    fn test_safe_commands() {
        let rep = check_safety("systemctl --user restart pipewire pipewire-pulse wireplumber");
        assert_ne!(rep.risk, RiskLevel::Destructive);
    }
}
