use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroFamily {
    Arch,
    Debian,
    Fedora,
    OpenSuse,
    Alpine,
    Void,
    Gentoo,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    Paru,
    Yay,
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Apk,
    Xbps,
    Emerge,
    Flatpak,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Paru => "paru",
            PackageManager::Yay => "yay",
            PackageManager::Pacman => "pacman",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
            PackageManager::Xbps => "xbps",
            PackageManager::Emerge => "emerge",
            PackageManager::Flatpak => "flatpak",
        }
    }

    pub fn install_cmd(&self, pkg: &str) -> String {
        match self {
            PackageManager::Paru => format!("paru -S {}", pkg),
            PackageManager::Yay => format!("yay -S {}", pkg),
            PackageManager::Pacman => format!("sudo pacman -S {}", pkg),
            PackageManager::Apt => format!("sudo apt install {}", pkg),
            PackageManager::Dnf => format!("sudo dnf install {}", pkg),
            PackageManager::Zypper => format!("sudo zypper install {}", pkg),
            PackageManager::Apk => format!("sudo apk add {}", pkg),
            PackageManager::Xbps => format!("sudo xbps-install -S {}", pkg),
            PackageManager::Emerge => format!("sudo emerge --ask {}", pkg),
            PackageManager::Flatpak => format!("flatpak install flathub {}", pkg),
        }
    }

    pub fn update_cmd(&self) -> String {
        match self {
            PackageManager::Paru => "paru -Syu".to_string(),
            PackageManager::Yay => "yay -Syu".to_string(),
            PackageManager::Pacman => "sudo pacman -Syu".to_string(),
            PackageManager::Apt => "sudo apt update && sudo apt upgrade".to_string(),
            PackageManager::Dnf => "sudo dnf upgrade".to_string(),
            PackageManager::Zypper => "sudo zypper update".to_string(),
            PackageManager::Apk => "sudo apk update && sudo apk upgrade".to_string(),
            PackageManager::Xbps => "sudo xbps-install -Su".to_string(),
            PackageManager::Emerge => "sudo emerge --sync && sudo emerge -avuDN @world".to_string(),
            PackageManager::Flatpak => "flatpak update".to_string(),
        }
    }

    pub fn remove_cmd(&self, pkg: &str) -> String {
        match self {
            PackageManager::Paru => format!("paru -Rns {}", pkg),
            PackageManager::Yay => format!("yay -Rns {}", pkg),
            PackageManager::Pacman => format!("sudo pacman -Rns {}", pkg),
            PackageManager::Apt => format!("sudo apt remove {}", pkg),
            PackageManager::Dnf => format!("sudo dnf remove {}", pkg),
            PackageManager::Zypper => format!("sudo zypper remove {}", pkg),
            PackageManager::Apk => format!("sudo apk del {}", pkg),
            PackageManager::Xbps => format!("sudo xbps-remove -R {}", pkg),
            PackageManager::Emerge => format!("sudo emerge --depclean {}", pkg),
            PackageManager::Flatpak => format!("flatpak uninstall {}", pkg),
        }
    }

    pub fn search_cmd(&self, query: &str) -> String {
        match self {
            PackageManager::Paru => format!("paru -Ss {}", query),
            PackageManager::Yay => format!("yay -Ss {}", query),
            PackageManager::Pacman => format!("pacman -Ss {}", query),
            PackageManager::Apt => format!("apt search {}", query),
            PackageManager::Dnf => format!("dnf search {}", query),
            PackageManager::Zypper => format!("zypper search {}", query),
            PackageManager::Apk => format!("apk search {}", query),
            PackageManager::Xbps => format!("xbps-query -Rs {}", query),
            PackageManager::Emerge => format!("emerge --search {}", query),
            PackageManager::Flatpak => format!("flatpak search {}", query),
        }
    }

    pub fn clean_cache_cmd(&self) -> String {
        match self {
            PackageManager::Paru => "paru -Scd".to_string(),
            PackageManager::Yay => "yay -Sc".to_string(),
            PackageManager::Pacman => "sudo pacman -Sc".to_string(),
            PackageManager::Apt => "sudo apt clean && sudo apt autoremove".to_string(),
            PackageManager::Dnf => "sudo dnf clean all".to_string(),
            PackageManager::Zypper => "sudo zypper clean".to_string(),
            PackageManager::Apk => "sudo apk cache clean".to_string(),
            PackageManager::Xbps => "sudo xbps-remove -O".to_string(),
            PackageManager::Emerge => "sudo eclean distfiles".to_string(),
            PackageManager::Flatpak => "flatpak uninstall --unused".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFactor {
    Laptop,
    Desktop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSystem {
    Pipewire,
    Pulseaudio,
    Alsa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InitSystem {
    Systemd,
    OpenRc,
    Runit,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayServer {
    Wayland(String),
    X11,
    Headless,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub cwd: PathBuf,
    pub files: Vec<String>,
    pub is_git_repo: bool,
    pub git_branch: Option<String>,
    pub git_modified_files: Vec<String>,
    pub os_id: String,
    pub distro_family: DistroFamily,
    pub pkg_manager: PackageManager,
    pub gpu_vendor: GpuVendor,
    pub form_factor: FormFactor,
    pub audio_system: AudioSystem,
    pub init_system: InitSystem,
    pub display_server: DisplayServer,
    pub has_asusctl: bool,
    pub shell: String,
    pub recent_history: Vec<String>,
}

impl Default for SystemContext {
    fn default() -> Self {
        Self {
            cwd: PathBuf::from("/home/user"),
            files: Vec::new(),
            is_git_repo: false,
            git_branch: None,
            git_modified_files: Vec::new(),
            os_id: "cachyos".to_string(),
            distro_family: DistroFamily::Arch,
            pkg_manager: PackageManager::Paru,
            gpu_vendor: GpuVendor::Nvidia,
            form_factor: FormFactor::Laptop,
            audio_system: AudioSystem::Pipewire,
            init_system: InitSystem::Systemd,
            display_server: DisplayServer::Wayland("hyprland".to_string()),
            has_asusctl: true,
            shell: "fish".to_string(),
            recent_history: Vec::new(),
        }
    }
}

static SANITIZER_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_sanitizers() -> &'static Vec<Regex> {
    SANITIZER_PATTERNS.get_or_init(|| {
        vec![
            // Key-value pairs: KEY=value or "key": "value"
            Regex::new(r#"(?i)(api[_-]?key|token|password|passwd|secret|auth|cookie|jwt|private[_-]?key)\s*([:=])\s*["']?([^\s"';&|]+)["']?"#).unwrap(),
            // Bearer tokens
            Regex::new(r#"(?i)bearer\s+([a-zA-Z0-9_\-\.]{10,})"#).unwrap(),
            // Basic auth URLs: https://user:pass@domain
            Regex::new(r#"https?://[^:\s]+:([^@\s]+)@"#).unwrap(),
            // Private keys
            Regex::new(r#"-----BEGIN[A-Z\s]+PRIVATE KEY-----[^-]+-----END[A-Z\s]+PRIVATE KEY-----"#).unwrap(),
        ]
    })
}

pub fn sanitize_text(input: &str) -> String {
    let mut result = input.to_string();
    let sanitizers = get_sanitizers();

    // Key-value pairs
    result = sanitizers[0]
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            let sep = &caps[2];
            format!("{}{}[REDACTED]", key, sep)
        })
        .to_string();

    // Bearer
    result = sanitizers[1]
        .replace_all(&result, "Bearer [REDACTED]")
        .to_string();

    // Basic auth url
    result = sanitizers[2]
        .replace_all(&result, "http://[REDACTED]@")
        .to_string();

    // Private keys
    result = sanitizers[3]
        .replace_all(&result, "[REDACTED PRIVATE KEY]")
        .to_string();

    result
}

// Hardware & distro static profile (cached across requests for 0ms overhead)
struct HardwareProfile {
    os_id: String,
    distro_family: DistroFamily,
    pkg_manager: PackageManager,
    gpu_vendor: GpuVendor,
    form_factor: FormFactor,
    audio_system: AudioSystem,
    init_system: InitSystem,
    display_server: DisplayServer,
    has_asusctl: bool,
}

static PROFILE: OnceLock<HardwareProfile> = OnceLock::new();

impl SystemContext {
    pub fn gather(
        cwd_override: Option<&Path>,
        shell_override: Option<&str>,
        history: Option<&[String]>,
    ) -> Self {
        let cwd = cwd_override
            .map(|p| p.to_path_buf())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("/home/prithibi"));

        let files = Self::scan_dir_files(&cwd);
        let (is_git_repo, git_branch, git_modified_files) = Self::inspect_git(&cwd);

        let prof = PROFILE.get_or_init(Self::detect_hardware_profile);
        let shell = shell_override.unwrap_or("fish").to_string();

        let sanitized_history = history
            .unwrap_or(&[])
            .iter()
            .take(5)
            .map(|cmd| sanitize_text(cmd))
            .collect();

        Self {
            cwd,
            files,
            is_git_repo,
            git_branch,
            git_modified_files,
            os_id: prof.os_id.clone(),
            distro_family: prof.distro_family,
            pkg_manager: prof.pkg_manager,
            gpu_vendor: prof.gpu_vendor,
            form_factor: prof.form_factor,
            audio_system: prof.audio_system,
            init_system: prof.init_system,
            display_server: prof.display_server.clone(),
            has_asusctl: prof.has_asusctl,
            shell,
            recent_history: sanitized_history,
        }
    }

    fn detect_hardware_profile() -> HardwareProfile {
        let (os_id, distro_family) = Self::detect_os_and_distro();
        let pkg_manager = Self::detect_pkg_manager(distro_family);
        let gpu_vendor = Self::detect_gpu();
        let form_factor = Self::detect_form_factor();
        let audio_system = Self::detect_audio();
        let init_system = Self::detect_init();
        let display_server = Self::detect_display();
        let has_asusctl = Self::has_binary("asusctl");

        HardwareProfile {
            os_id,
            distro_family,
            pkg_manager,
            gpu_vendor,
            form_factor,
            audio_system,
            init_system,
            display_server,
            has_asusctl,
        }
    }

    fn scan_dir_files(dir: &Path) -> Vec<String> {
        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten().take(60) {
                if let Ok(name) = entry.file_name().into_string() {
                    if !name.starts_with('.') || name == ".git" {
                        list.push(name);
                    }
                }
            }
        }
        list
    }

    fn inspect_git(dir: &Path) -> (bool, Option<String>, Vec<String>) {
        let mut curr = dir;
        let mut has_git = false;
        loop {
            if curr.join(".git").exists() {
                has_git = true;
                break;
            }
            match curr.parent() {
                Some(p) => curr = p,
                None => break,
            }
        }

        if !has_git {
            return (false, None, Vec::new());
        }

        let branch_output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(dir)
            .output();

        let branch = match branch_output {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            }
            _ => None,
        };

        let status_output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .arg("-unormal")
            .current_dir(dir)
            .output();

        let mut modified = Vec::new();
        if let Ok(out) = status_output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().take(15) {
                    if line.len() > 3 {
                        modified.push(line[3..].to_string());
                    }
                }
            }
        }

        (true, branch, modified)
    }

    fn detect_os_and_distro() -> (String, DistroFamily) {
        let mut os_id = "cachyos".to_string();
        let mut id_like = String::new();

        if let Ok(contents) = fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if let Some(val) = line.strip_prefix("ID=") {
                    os_id = val.trim_matches('"').trim_matches('\'').to_lowercase();
                } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
                    id_like = val.trim_matches('"').trim_matches('\'').to_lowercase();
                }
            }
        }

        let combined = format!("{} {}", os_id, id_like);

        let family = if combined.contains("arch") || combined.contains("cachyos") || combined.contains("manjaro") || combined.contains("endeavour") || combined.contains("artix") {
            DistroFamily::Arch
        } else if combined.contains("ubuntu") || combined.contains("debian") || combined.contains("mint") || combined.contains("pop") || combined.contains("kali") || combined.contains("raspbian") {
            DistroFamily::Debian
        } else if combined.contains("fedora") || combined.contains("rhel") || combined.contains("centos") || combined.contains("rocky") || combined.contains("alma") {
            DistroFamily::Fedora
        } else if combined.contains("suse") {
            DistroFamily::OpenSuse
        } else if combined.contains("alpine") {
            DistroFamily::Alpine
        } else if combined.contains("void") {
            DistroFamily::Void
        } else if combined.contains("gentoo") {
            DistroFamily::Gentoo
        } else {
            DistroFamily::Generic
        };

        (os_id, family)
    }

    fn detect_pkg_manager(family: DistroFamily) -> PackageManager {
        // Check for specific modern helpers first
        if Self::has_binary("paru") {
            return PackageManager::Paru;
        }
        if Self::has_binary("yay") {
            return PackageManager::Yay;
        }

        match family {
            DistroFamily::Arch => {
                if Self::has_binary("pacman") {
                    PackageManager::Pacman
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::Debian => {
                if Self::has_binary("apt") || Self::has_binary("apt-get") {
                    PackageManager::Apt
                } else {
                    PackageManager::Flatpak
                }
            }
            DistroFamily::Fedora => {
                if Self::has_binary("dnf") {
                    PackageManager::Dnf
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::OpenSuse => {
                if Self::has_binary("zypper") {
                    PackageManager::Zypper
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::Alpine => {
                if Self::has_binary("apk") {
                    PackageManager::Apk
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::Void => {
                if Self::has_binary("xbps-install") {
                    PackageManager::Xbps
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::Gentoo => {
                if Self::has_binary("emerge") {
                    PackageManager::Emerge
                } else {
                    PackageManager::Apt
                }
            }
            DistroFamily::Generic => {
                if Self::has_binary("pacman") {
                    PackageManager::Pacman
                } else if Self::has_binary("apt") {
                    PackageManager::Apt
                } else if Self::has_binary("dnf") {
                    PackageManager::Dnf
                } else if Self::has_binary("zypper") {
                    PackageManager::Zypper
                } else if Self::has_binary("apk") {
                    PackageManager::Apk
                } else {
                    PackageManager::Flatpak
                }
            }
        }
    }

    fn detect_gpu() -> GpuVendor {
        // Fast sysfs inspection of DRM PCI devices (zero child processes)
        let mut vendors = Vec::new();
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("card") && !name_str.contains('-') {
                    let vendor_path = entry.path().join("device/vendor");
                    if let Ok(content) = fs::read_to_string(vendor_path) {
                        let id = content.trim().to_lowercase();
                        vendors.push(id);
                    }
                }
            }
        }

        // Prioritize dedicated GPUs: NVIDIA (0x10de) or AMD (0x1002)
        if vendors.iter().any(|v| v == "0x10de") {
            return GpuVendor::Nvidia;
        }
        if vendors.iter().any(|v| v == "0x1002") {
            return GpuVendor::Amd;
        }
        if vendors.iter().any(|v| v == "0x8086") {
            return GpuVendor::Intel;
        }

        // Fallback: check if nvidia-smi exists in PATH
        if Self::has_binary("nvidia-smi") {
            return GpuVendor::Nvidia;
        }
        if Self::has_binary("rocm-smi") || Self::has_binary("radeontop") {
            return GpuVendor::Amd;
        }
        if Self::has_binary("intel_gpu_top") {
            return GpuVendor::Intel;
        }

        GpuVendor::Generic
    }

    fn detect_form_factor() -> FormFactor {
        // 1. Check if battery device exists in /sys/class/power_supply
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("BAT") {
                    return FormFactor::Laptop;
                }
                let type_path = entry.path().join("type");
                if let Ok(content) = fs::read_to_string(type_path) {
                    if content.trim().eq_ignore_ascii_case("Battery") {
                        return FormFactor::Laptop;
                    }
                }
            }
        }

        // 2. Check DMI chassis type
        if let Ok(content) = fs::read_to_string("/sys/class/dmi/id/chassis_type") {
            let t = content.trim();
            // 8: Portable, 9: Laptop, 10: Notebook, 11: Hand Held, 14: Sub Notebook, 30: Tablet, 31: Convertible, 32: Detachable
            if matches!(t, "8" | "9" | "10" | "11" | "14" | "30" | "31" | "32") {
                return FormFactor::Laptop;
            }
        }

        FormFactor::Desktop
    }

    fn detect_audio() -> AudioSystem {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let pipewire_socket = format!("{}/pipewire-0", runtime_dir);
        if Path::new(&pipewire_socket).exists() || Self::has_binary("wpctl") {
            return AudioSystem::Pipewire;
        }

        let pulse_socket = format!("{}/pulse/native", runtime_dir);
        if Path::new(&pulse_socket).exists() || Self::has_binary("pactl") {
            return AudioSystem::Pulseaudio;
        }

        AudioSystem::Alsa
    }

    fn detect_init() -> InitSystem {
        if Path::new("/run/systemd/system").exists() {
            InitSystem::Systemd
        } else if Path::new("/run/openrc").exists() {
            InitSystem::OpenRc
        } else if Path::new("/run/runit").exists() {
            InitSystem::Runit
        } else {
            InitSystem::Generic
        }
    }

    fn detect_display() -> DisplayServer {
        if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
            if !wayland.is_empty() {
                let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
                if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() || desktop.contains("hyprland") {
                    return DisplayServer::Wayland("hyprland".to_string());
                }
                if std::env::var("SWAYSOCK").is_ok() || desktop.contains("sway") {
                    return DisplayServer::Wayland("sway".to_string());
                }
                if desktop.contains("gnome") {
                    return DisplayServer::Wayland("gnome".to_string());
                }
                if desktop.contains("kde") {
                    return DisplayServer::Wayland("kde".to_string());
                }
                return DisplayServer::Wayland("generic".to_string());
            }
        }

        if let Ok(disp) = std::env::var("DISPLAY") {
            if !disp.is_empty() {
                return DisplayServer::X11;
            }
        }

        DisplayServer::Headless
    }

    fn has_binary(bin: &str) -> bool {
        // Fast path check without spawning subshell
        let common_paths = [
            "/usr/bin",
            "/usr/local/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ];
        for p in &common_paths {
            if Path::new(p).join(bin).exists() {
                return true;
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            if Path::new(&home).join(".local/bin").join(bin).exists() {
                return true;
            }
            if Path::new(&home).join(".cargo/bin").join(bin).exists() {
                return true;
            }
        }

        // Check PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if dir.join(bin).exists() {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_secrets() {
        let input = "export OPENAI_API_KEY=sk-proj-1234567890abcdef and token: secret_value_99";
        let sanitized = sanitize_text(input);
        assert!(!sanitized.contains("sk-proj"));
        assert!(!sanitized.contains("secret_value_99"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_bearer() {
        let input = "curl -H 'Authorization: Bearer mysecrettoken1234567' https://api.example.com";
        let sanitized = sanitize_text(input);
        assert!(!sanitized.contains("mysecrettoken1234567"));
        assert!(sanitized.contains("Bearer [REDACTED]"));
    }

    #[test]
    fn test_package_manager_methods() {
        let pacman = PackageManager::Pacman;
        assert_eq!(pacman.install_cmd("curl"), "sudo pacman -S curl");
        assert_eq!(pacman.update_cmd(), "sudo pacman -Syu");
        assert_eq!(pacman.remove_cmd("curl"), "sudo pacman -Rns curl");

        let apt = PackageManager::Apt;
        assert_eq!(apt.install_cmd("curl"), "sudo apt install curl");
        assert_eq!(apt.update_cmd(), "sudo apt update && sudo apt upgrade");
        assert_eq!(apt.remove_cmd("curl"), "sudo apt remove curl");

        let dnf = PackageManager::Dnf;
        assert_eq!(dnf.install_cmd("curl"), "sudo dnf install curl");
        assert_eq!(dnf.update_cmd(), "sudo dnf upgrade");

        let zypper = PackageManager::Zypper;
        assert_eq!(zypper.install_cmd("curl"), "sudo zypper install curl");

        let apk = PackageManager::Apk;
        assert_eq!(apk.install_cmd("curl"), "sudo apk add curl");

        let xbps = PackageManager::Xbps;
        assert_eq!(xbps.install_cmd("curl"), "sudo xbps-install -S curl");
    }

    #[test]
    fn test_gather_context() {
        let ctx = SystemContext::gather(None, Some("fish"), None);
        assert!(!ctx.os_id.is_empty());
        assert_eq!(ctx.shell, "fish");
    }
}
