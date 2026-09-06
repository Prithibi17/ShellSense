use crate::context::{AudioSystem, DistroFamily, FormFactor, GpuVendor, InitSystem, DisplayServer, SystemContext};
use crate::fuzzy::{clean_intent, classify_action_token, fuzzy_find_best, matches_fuzzy, ActionVerb};
use crate::protocol::{CandidateSuggestion, RiskLevel};
use regex::Regex;

pub fn match_deterministic(input: &str, ctx: &SystemContext) -> Vec<CandidateSuggestion> {
    let mut suggestions = Vec::new();
    let lower = input.trim().to_lowercase();
    if lower.is_empty() {
        return suggestions;
    }

    let (cleaned, tokens) = clean_intent(&lower);

    // 1. Direct Shell Typos & Quick Directory Navigation
    if let Some(sugs) = match_shell_typos_and_nav(&lower) {
        return sugs;
    }

    // 2. Standard Command Flags & Smart Parameter Completion
    if let Some(sugs) = match_command_tool_flags(&lower) {
        return sugs;
    }

    // 3. Developer Tools, Python venv, Rust, Docker & Shell Environment
    if let Some(sugs) = match_developer_and_env(&lower, ctx) {
        return sugs;
    }

    // 4. Path-Aware Archive Extraction
    if lower.starts_with("extract") || lower.starts_with("unzip") || lower.starts_with("untar") || lower.starts_with("decompress") {
        if let Some(sug) = match_archive_extraction(&lower, ctx) {
            suggestions.push(sug);
            return suggestions;
        }
    }

    // 5. Path-Aware Project Execution & Building
    if let Some(sugs) = match_path_aware_projects(&lower, ctx) {
        return sugs;
    }

    // 6. Git Context Operations
    if ctx.is_git_repo || lower.starts_with("git") {
        if let Some(sugs) = match_git_workflow(&lower, ctx) {
            return sugs;
        }
    }

    // 7. Generalized Semantic Intent Engine (typo-tolerant, word-order invariant, multi-distro)
    if let Some(sugs) = match_semantic_intent(&lower, &cleaned, &tokens, ctx) {
        return sugs;
    }

    // 8. Power, Reboot, Shutdown & Session Controls (Legacy fallback)
    if let Some(sugs) = match_system_power_and_session(&lower, ctx) {
        return sugs;
    }

    // 9. Universal Package Management (Legacy fallback)
    if let Some(sugs) = match_package_management(&lower, ctx) {
        return sugs;
    }

    // 10. Flatpak Management (Legacy fallback)
    if let Some(sugs) = match_flatpak_operations(&lower) {
        return sugs;
    }

    // 11. Hardware, GPU (NVIDIA / AMD / Intel), Battery, Backlight & Audio (Legacy fallback)
    if let Some(sugs) = match_hardware_and_laptop(&lower, ctx) {
        return sugs;
    }

    // 12. Services & System Administration (systemd / OpenRC / runit aware)
    if let Some(sugs) = match_services_and_init(&lower, ctx) {
        return sugs;
    }

    // 13. Network, Ports, IP & Connectivity (Legacy fallback)
    if let Some(sugs) = match_network_and_ports(&lower) {
        return sugs;
    }

    // 14. Files, Storage, Disks, Search & Permissions (Legacy fallback)
    if let Some(sugs) = match_files_and_storage(&lower, ctx) {
        return sugs;
    }

    // 15. Process Management, Performance & Memory (Legacy fallback)
    if let Some(sugs) = match_processes_and_performance(&lower) {
        return sugs;
    }

    // 16. Desktop Environment & Window Manager (Hyprland, Sway, GNOME, KDE, X11)
    if let Some(sugs) = match_desktop_and_window_manager(&lower, ctx) {
        return sugs;
    }

    // 17. Distro-Specific Extras (Btrfs, Snapper, CachyOS)
    if ctx.distro_family == DistroFamily::Arch {
        if let Some(sugs) = match_btrfs_and_cachyos(&lower) {
            return sugs;
        }
    }

    suggestions
}

// -----------------------------------------------------------------------------
// 0. Generalized Semantic Intent Engine
// -----------------------------------------------------------------------------
fn match_semantic_intent(
    _lower: &str,
    cleaned: &str,
    tokens: &[String],
    ctx: &SystemContext,
) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // 0. ShellSense Self-Commands (Update, Status)
    if tokens.iter().any(|t| t == "shellsense" || t == "ss") {
        if tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Update)) {
            sugs.push(sug("ss update", "Update ShellSense to the latest GitHub release", 0.99, RiskLevel::Low, None, "ShellSense"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Query)) || tokens.iter().any(|t| t == "status") {
            sugs.push(sug("ss status", "Check ShellSense daemon status and AI engine health", 0.99, RiskLevel::Low, None, "ShellSense"));
            return Some(sugs);
        }
    }

    // 1. System Session & Power Controls (Reboot, Shutdown, Sleep, Lock, Logout)
    let is_systemd = ctx.init_system == InitSystem::Systemd;
    let is_reboot = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::PowerReboot))
        || (tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Restart))
            && tokens.iter().any(|t| ["pc", "laptop", "computer", "system", "machine", "box"].contains(&t.as_str())))
        || cleaned == "reboot" || cleaned == "rebot" || cleaned == "restart system";
    if is_reboot {
        let cmd = if is_systemd { "systemctl reboot" } else { "sudo reboot" };
        sugs.push(sug(cmd, "Reboot and restart the operating system", 0.99, RiskLevel::Medium, Some("Reboots system immediately".to_string()), "System"));
        return Some(sugs);
    }

    let is_shutdown = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::PowerShutdown))
        || (tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Kill))
            && tokens.iter().any(|t| ["pc", "laptop", "computer", "system", "machine", "power"].contains(&t.as_str())))
        || (tokens.iter().any(|t| t == "turn" || t == "shut") && tokens.iter().any(|t| t == "off" || t == "of" || t == "down"))
        || cleaned == "shutdown" || cleaned == "poweroff" || cleaned == "power off" || cleaned == "pwerof" || cleaned == "turn off" || cleaned == "shut down";
    if is_shutdown {
        let cmd = if is_systemd { "systemctl poweroff" } else { "sudo poweroff" };
        sugs.push(sug(cmd, "Safely shut down and power off the machine", 0.99, RiskLevel::Destructive, Some("Powers off machine immediately".to_string()), "System"));
        return Some(sugs);
    }

    let is_sleep = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::PowerSleep))
        || cleaned == "sleep" || cleaned == "suspend";
    if is_sleep {
        let cmd = if is_systemd { "systemctl suspend" } else { "sudo zzz || sudo pm-suspend" };
        sugs.push(sug(cmd, "Suspend system into low-power RAM sleep mode", 0.99, RiskLevel::Low, None, "System"));
        return Some(sugs);
    }

    let is_hibernate = cleaned == "hibernate" || cleaned == "hibarnate" || cleaned == "hibernate pc";
    if is_hibernate {
        let cmd = if is_systemd { "systemctl hibernate" } else { "sudo ZZZ || sudo pm-hibernate" };
        sugs.push(sug(cmd, "Hibernate system state onto disk swap and power down", 0.98, RiskLevel::Medium, None, "System"));
        return Some(sugs);
    }

    let is_lock = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::PowerLock))
        || cleaned == "lock" || cleaned == "lock screen" || cleaned == "lockscreen";
    if is_lock {
        let cmd = match &ctx.display_server {
            DisplayServer::Wayland(wm) if wm == "hyprland" => "hyprlock",
            DisplayServer::Wayland(wm) if wm == "sway" => "swaylock",
            DisplayServer::Wayland(wm) if wm == "gnome" => "loginctl lock-session",
            DisplayServer::Wayland(wm) if wm == "kde" => "loginctl lock-session",
            DisplayServer::X11 => "xflock4 || i3lock || loginctl lock-session",
            _ => "loginctl lock-session",
        };
        sugs.push(sug(cmd, "Lock active desktop screen session", 0.99, RiskLevel::Low, None, "Desktop"));
        return Some(sugs);
    }

    let is_logout = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::PowerLogout))
        || cleaned == "logout" || cleaned == "exit desktop" || cleaned == "quit desktop";
    if is_logout {
        let cmd = match &ctx.display_server {
            DisplayServer::Wayland(wm) if wm == "hyprland" => "hyprctl dispatch exit",
            DisplayServer::Wayland(wm) if wm == "sway" => "swaymsg exit",
            _ => "loginctl terminate-user $USER",
        };
        sugs.push(sug(cmd, "Exit desktop session and return to display manager", 0.98, RiskLevel::Medium, None, "Desktop"));
        return Some(sugs);
    }

    // 2. Hardware: GPU (NVIDIA / AMD / Intel)
    let is_gpu = tokens.iter().any(|t| {
        let s = t.as_str();
        s == "gpu" || s == "gpui" || s == "nvidia" || s == "nvda" || s == "nvdia" || s == "radeon"
            || s == "amd" || s == "geforce" || s == "rtx" || s == "gtx" || s == "vram"
            || matches_fuzzy(s, "nvidia", 0.75) || matches_fuzzy(s, "graphics", 0.75)
    });
    if is_gpu {
        match ctx.gpu_vendor {
            GpuVendor::Nvidia => {
                sugs.push(sug("nvidia-smi", "Display NVIDIA GPU utilization, temperature, and VRAM allocation", 0.99, RiskLevel::Low, None, "Hardware"));
                sugs.push(sug("watch -n 1 nvidia-smi", "Monitor NVIDIA GPU stats live every second", 0.94, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Amd => {
                sugs.push(sug("radeontop", "Monitor AMD Radeon GPU utilization and VRAM in real-time", 0.99, RiskLevel::Low, None, "Hardware"));
                sugs.push(sug("rocm-smi", "Query AMD ROCm GPU clocks, temperature, and power metrics", 0.94, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Intel => {
                sugs.push(sug("sudo intel_gpu_top", "Monitor Intel Arc and integrated GPU engine render metrics", 0.99, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Generic => {
                sugs.push(sug("lspci -nnk | grep -A4 -E \"VGA|3D|Display\"", "Inspect graphics hardware and active kernel drivers", 0.98, RiskLevel::Low, None, "Hardware"));
            }
        }
        sugs.push(sug("lspci -nnk | grep -A4 -E \"VGA|3D|Display\"", "Inspect graphics controller and active kernel drivers", 0.88, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // 3. Audio / Sound (PipeWire, PulseAudio, ALSA)
    let is_audio = tokens.iter().any(|t| {
        let s = t.as_str();
        s == "audio" || s == "audi" || s == "sound" || s == "snd" || s == "pipewire"
            || s == "pipwire" || s == "pulseaudio" || s == "pulse" || s == "wireplumber"
            || matches_fuzzy(s, "audio", 0.75) || matches_fuzzy(s, "sound", 0.75)
    });
    if is_audio {
        let is_query_action = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Query))
            && !tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Restart) || classify_action_token(t) == Some(ActionVerb::Fix));
        if is_query_action {
            let (cmd, desc) = match ctx.audio_system {
                AudioSystem::Pipewire => ("wpctl status", "Display active PipeWire audio endpoints and volume"),
                AudioSystem::Pulseaudio => ("pactl list sinks short", "List active PulseAudio audio output sinks"),
                _ => ("alsamixer", "Open interactive ALSA sound mixer interface"),
            };
            sugs.push(sug(cmd, desc, 0.98, RiskLevel::Low, None, "Audio"));
            return Some(sugs);
        } else {
            let (cmd, desc) = match ctx.audio_system {
                AudioSystem::Pipewire => (
                    "systemctl --user restart pipewire pipewire-pulse wireplumber",
                    "Restart modern PipeWire multimedia sound server & session manager",
                ),
                AudioSystem::Pulseaudio => (
                    "pulseaudio -k && pulseaudio --start",
                    "Kill and respawn PulseAudio audio daemon",
                ),
                _ => (
                    "sudo alsactl restore",
                    "Restore ALSA soundcard driver states",
                ),
            };
            sugs.push(sug(cmd, desc, 0.99, RiskLevel::Low, None, "Audio"));
            return Some(sugs);
        }
    }

    // 4. Laptop Battery, Fan Profiles & Backlight
    let is_battery = tokens.iter().any(|t| {
        let s = t.as_str();
        s == "battery" || s == "bat" || s == "batry" || s == "charge" || s == "charging"
            || matches_fuzzy(s, "battery", 0.75)
    });
    if is_battery {
        if tokens.iter().any(|t| ["limit", "80", "care", "health"].contains(&t.as_str())) && ctx.has_asusctl {
            sugs.push(sug("asusctl -c 80", "Set battery charge threshold to 80% to preserve lithium health", 0.99, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
        sugs.push(sug("upower -i $(upower -e | grep 'BAT')", "Display detailed battery health, discharge rate, and cycle count", 0.99, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("cat /sys/class/power_supply/BAT*/capacity 2>/dev/null || acpi -b", "Read current battery percentage directly from ACPI sysfs", 0.94, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // Fan / Power profiles
    let is_fan_profile = tokens.iter().any(|t| ["fan", "turbo", "profile", "performance", "powersave"].contains(&t.as_str()));
    if is_fan_profile {
        if tokens.iter().any(|t| ["turbo", "performance", "high", "max"].contains(&t.as_str())) {
            if ctx.has_asusctl {
                sugs.push(sug("asusctl profile -P Performance", "Activate ASUS ROG/TUF Performance fan & thermal profile", 0.99, RiskLevel::Low, None, "Hardware"));
            } else {
                sugs.push(sug("powerprofilesctl set performance", "Engage high performance power profile via system power-profiles-daemon", 0.99, RiskLevel::Low, None, "Hardware"));
            }
            return Some(sugs);
        }
        if tokens.iter().any(|t| ["quiet", "silent", "powersave", "eco"].contains(&t.as_str())) {
            if ctx.has_asusctl {
                sugs.push(sug("asusctl profile -P Quiet", "Activate ASUS ROG/TUF Quiet fan & thermal profile", 0.99, RiskLevel::Low, None, "Hardware"));
            } else {
                sugs.push(sug("powerprofilesctl set power-saver", "Engage battery-saving power profile via power-profiles-daemon", 0.99, RiskLevel::Low, None, "Hardware"));
            }
            return Some(sugs);
        }
    }

    // 5. Ports & Network Connections
    if let Some((port, is_kill)) = parse_port_intent(cleaned, tokens) {
        if is_kill {
            sugs.push(sug(
                &format!("kill -9 $(lsof -t -i:{})", port),
                &format!("Forcefully terminate process listening on port {}", port),
                0.99,
                RiskLevel::Destructive,
                Some(format!("⚠ Terminates all processes bound to port {}.", port)),
                "Network",
            ));
            sugs.push(sug(
                &format!("fuser -k {}/tcp", port),
                &format!("Kill any process occupying TCP port {}", port),
                0.95,
                RiskLevel::Destructive,
                Some(format!("⚠ Kills process on TCP port {}.", port)),
                "Network",
            ));
        } else {
            sugs.push(sug(
                &format!("ss -ltnp | grep ':{}'", port),
                &format!("Find listening process bound to TCP port {}", port),
                0.99,
                RiskLevel::Low,
                None,
                "Network",
            ));
            sugs.push(sug(
                &format!("fuser {}/tcp", port),
                &format!("List process ID occupying TCP port {}", port),
                0.92,
                RiskLevel::Low,
                None,
                "Network",
            ));
        }
        return Some(sugs);
    }

    let is_ports_general = tokens.iter().any(|t| ["port", "ports", "prt", "prts", "sockets"].contains(&t.as_str()))
        && (tokens.iter().any(|t| ["open", "listening", "listen", "check", "show", "list"].contains(&t.as_str())) || tokens.len() == 1);
    if is_ports_general {
        sugs.push(sug("ss -tulwn", "List all listening TCP and UDP sockets and port numbers", 0.99, RiskLevel::Low, None, "Network"));
        sugs.push(sug("sudo lsof -i -P -n | grep LISTEN", "Inspect all listening sockets with owning process names", 0.94, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    // IP Address & WiFi
    let is_public_ip = tokens.iter().any(|t| t == "public" || t == "external" || t == "wan")
        && tokens.iter().any(|t| ["ip", "address"].contains(&t.as_str()));
    if is_public_ip {
        sugs.push(sug("curl -s ifconfig.me", "Query and display current public WAN IP address", 0.99, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    let is_ip = tokens.iter().any(|t| ["ip", "myip"].contains(&t.as_str()))
        || (tokens.iter().any(|t| ["local", "network", "lan", "net"].contains(&t.as_str())) && tokens.iter().any(|t| t == "ip"));
    if is_ip {
        sugs.push(sug("ip -br a", "Show brief list of network interfaces and assigned local IP addresses", 0.99, RiskLevel::Low, None, "Network"));
        sugs.push(sug("curl -s ifconfig.me", "Query and display current public WAN IP address", 0.95, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    let is_wifi = tokens.iter().any(|t| ["wifi", "wi-fi", "wlan"].contains(&t.as_str()));
    if is_wifi {
        sugs.push(sug("nmcli dev wifi list", "Scan and list available Wi-Fi networks with signal strength", 0.99, RiskLevel::Low, None, "Network"));
        return Some(sugs);
    }

    // 6. Storage, Disks, Memory & CPU
    let is_disk = tokens.iter().any(|t| {
        let s = t.as_str();
        s == "disk" || s == "disks" || s == "dsk" || s == "storage" || s == "storag"
            || s == "space" || s == "drive" || s == "drives" || s == "hdd" || s == "ssd"
            || matches_fuzzy(s, "storage", 0.75)
    });
    if is_disk {
        if tokens.iter().any(|t| ["large", "big", "heavy", "biggest", "du"].contains(&t.as_str())) {
            sugs.push(sug("du -sh * | sort -h", "Calculate directory sizes in current path and sort ascending", 0.98, RiskLevel::Low, None, "Storage"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| ["disks", "drives", "partitions", "lsblk"].contains(&t.as_str())) || cleaned == "show disks" {
            sugs.push(sug("lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS", "Inspect block devices, partitions, and mountpoints", 0.99, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("df -h", "Display filesystem disk space usage in human-readable gigabytes", 0.96, RiskLevel::Low, None, "Storage"));
        } else {
            sugs.push(sug("df -h", "Display filesystem disk space usage in human-readable gigabytes", 0.99, RiskLevel::Low, None, "Storage"));
            sugs.push(sug("lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS", "Inspect block devices, partitions, and mountpoints", 0.96, RiskLevel::Low, None, "Storage"));
        }
        return Some(sugs);
    }

    let is_memory = tokens.iter().any(|t| ["ram", "memory", "mem", "swap"].contains(&t.as_str()));
    if is_memory {
        sugs.push(sug("free -h", "Display amount of free and used physical memory and swap", 0.99, RiskLevel::Low, None, "Memory"));
        sugs.push(sug("btop", "Launch modern interactive resource monitor", 0.94, RiskLevel::Low, None, "Processes"));
        return Some(sugs);
    }

    let is_cpu = tokens.iter().any(|t| ["cpu", "processor", "proc", "processes", "tasks", "top"].contains(&t.as_str()));
    if is_cpu {
        sugs.push(sug("ps aux --sort=-%cpu | head -n 10", "List top 10 CPU-consuming processes", 0.98, RiskLevel::Low, None, "Processes"));
        sugs.push(sug("btop", "Launch modern interactive CPU and process monitor", 0.94, RiskLevel::Low, None, "Processes"));
        return Some(sugs);
    }

    // 7. Universal Package Management (cross-distro, typo-tolerant, word-order invariant)
    let pm = ctx.pkg_manager;

    // System Updates
    let is_sys_update = tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Update))
        && (tokens.len() == 1 || tokens.iter().any(|t| ["system", "os", "linux", "packages", "distro", "all", "pc"].contains(&t.as_str())));
    if is_sys_update {
        sugs.push(sug(&pm.update_cmd(), &format!("Upgrade all system packages using {}", pm.name()), 0.99, RiskLevel::Low, None, "Packages"));
        if ctx.distro_family == DistroFamily::Arch && pm != crate::context::PackageManager::Paru {
            sugs.push(sug("paru -Syu", "Upgrade both official and AUR packages seamlessly", 0.94, RiskLevel::Low, None, "AUR"));
        }
        return Some(sugs);
    }

    // Clean cache
    let is_clean_cache = tokens.iter().any(|t| ["cache", "caches", "pkgcache"].contains(&t.as_str()))
        && (tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Clean) || classify_action_token(t) == Some(ActionVerb::Remove)) || tokens.len() == 1);
    if is_clean_cache {
        sugs.push(sug(&pm.clean_cache_cmd(), &format!("Clean cached package files using {}", pm.name()), 0.98, RiskLevel::Low, None, "Packages"));
        return Some(sugs);
    }

    // Clean orphans
    let is_clean_orphans = tokens.iter().any(|t| ["orphans", "orphan", "unused", "unneeded"].contains(&t.as_str()) || t == "autoremove");
    if is_clean_orphans {
        let cmd = match ctx.distro_family {
            DistroFamily::Arch => "sudo pacman -Rns (pacman -Qtdq)",
            DistroFamily::Debian => "sudo apt autoremove",
            DistroFamily::Fedora => "sudo dnf autoremove",
            DistroFamily::OpenSuse => "sudo zypper packages --orphaned",
            DistroFamily::Void => "sudo xbps-remove -o",
            DistroFamily::Alpine => "sudo apk cache clean",
            _ => "sudo apt autoremove",
        };
        sugs.push(sug(cmd, "Remove unneeded orphaned dependencies", 0.98, RiskLevel::Destructive, Some("⚠ Removes unused package dependencies. Review list before confirming.".to_string()), "Packages"));
        return Some(sugs);
    }

    // Arch keyring & mirrors & lock
    if ctx.distro_family == DistroFamily::Arch {
        if tokens.iter().any(|t| ["keys", "keyring", "gpg"].contains(&t.as_str())) && tokens.iter().any(|t| ["fix", "repair", "update", "refresh"].contains(&t.as_str())) {
            sugs.push(sug("sudo pacman -Sy archlinux-keyring cachyos-keyring && sudo pacman-key --refresh-keys", "Refresh and populate Arch Linux and CachyOS cryptographic GPG keyrings", 0.99, RiskLevel::Medium, None, "Pacman"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| ["mirror", "mirrors"].contains(&t.as_str())) && tokens.iter().any(|t| ["update", "fastest", "rate"].contains(&t.as_str())) {
            sugs.push(sug("sudo cachyos-rate-mirrors", "Benchmark and update CachyOS and Arch package mirrors", 0.98, RiskLevel::Low, None, "Pacman"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| ["lock", "db.lck"].contains(&t.as_str())) && tokens.iter().any(|t| ["unlock", "remove", "rm", "pacman"].contains(&t.as_str())) {
            sugs.push(sug("sudo rm -f /var/lib/pacman/db.lck", "Remove stale pacman database lock file", 0.99, RiskLevel::Medium, None, "Pacman"));
            return Some(sugs);
        }
    }

    // List installed packages
    if tokens.iter().any(|t| ["list", "show", "all"].contains(&t.as_str())) && tokens.iter().any(|t| ["packages", "installed", "pkgs"].contains(&t.as_str())) {
        let cmd = match ctx.distro_family {
            DistroFamily::Arch => "pacman -Qe",
            DistroFamily::Debian => "apt list --installed",
            DistroFamily::Fedora => "dnf list installed",
            DistroFamily::OpenSuse => "zypper search -i",
            DistroFamily::Alpine => "apk info",
            DistroFamily::Void => "xbps-query -l",
            _ => "apt list --installed",
        };
        sugs.push(sug(cmd, "List all explicitly installed packages on the system", 0.98, RiskLevel::Low, None, "Packages"));
        return Some(sugs);
    }

    // Arbitrary package install
    if let Some(pkg) = parse_install_intent(tokens) {
        if let Some((cmd, desc, cat)) = resolve_package_command(&pkg, ctx) {
            sugs.push(sug(&cmd, &desc, 0.98, RiskLevel::Low, None, &cat));
            return Some(sugs);
        }
    }

    // Arbitrary package remove
    if let Some(pkg) = parse_remove_intent(tokens) {
        if let Some((cmd, desc, cat)) = resolve_package_remove(&pkg, ctx) {
            sugs.push(sug(
                &cmd,
                &desc,
                0.98,
                RiskLevel::Destructive,
                Some("⚠ Removes package and related dependencies. Review list before confirming.".to_string()),
                &cat,
            ));
            return Some(sugs);
        }
    }

    // Arbitrary package search
    if let Some(pkg) = parse_find_package_intent(tokens) {
        sugs.push(sug(&pm.search_cmd(&pkg), &format!("Search {} repositories for '{}'", pm.name(), pkg), 0.98, RiskLevel::Low, None, "Packages"));
        return Some(sugs);
    }

    // 8. Flatpaks
    if tokens.iter().any(|t| t == "flatpak" || t == "flatpaks") {
        if tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Update)) {
            sugs.push(sug("flatpak update", "Update all installed Flatpak applications and runtimes", 0.99, RiskLevel::Low, None, "Flatpak"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Query) || t == "list") {
            sugs.push(sug("flatpak list --app", "List all installed Flatpak desktop applications", 0.99, RiskLevel::Low, None, "Flatpak"));
            return Some(sugs);
        }
        if tokens.iter().any(|t| classify_action_token(t) == Some(ActionVerb::Clean) || t == "unused") {
            sugs.push(sug("flatpak uninstall --unused", "Uninstall unused Flatpak runtimes to reclaim space", 0.98, RiskLevel::Low, None, "Flatpak"));
            return Some(sugs);
        }
    }

    // 9. Quick Directory Navigation (cd download, cd documents, etc.)
    if let Some(sug_nav) = match_semantic_navigation(tokens) {
        sugs.push(sug_nav);
        return Some(sugs);
    }

    None
}

fn match_semantic_navigation(tokens: &[String]) -> Option<CandidateSuggestion> {
    if tokens.is_empty() || tokens.len() > 3 {
        return None;
    }
    for t in tokens {
        match t.as_str() {
            "downloads" | "download" => return Some(sug("cd ~/Downloads", "Navigate to user Downloads folder", 0.99, RiskLevel::Low, None, "Shell")),
            "documents" | "doc" | "docs" => return Some(sug("cd ~/Documents", "Navigate to user Documents folder", 0.99, RiskLevel::Low, None, "Shell")),
            "pictures" | "pics" | "photos" => return Some(sug("cd ~/Pictures", "Navigate to user Pictures folder", 0.99, RiskLevel::Low, None, "Shell")),
            "config" => return Some(sug("cd ~/.config", "Navigate to user XDG configuration folder", 0.99, RiskLevel::Low, None, "Shell")),
            "projects" | "dev" => return Some(sug("cd ~/Projects", "Navigate to user Projects directory", 0.99, RiskLevel::Low, None, "Shell")),
            "desktop" => return Some(sug("cd ~/Desktop", "Navigate to user Desktop folder", 0.99, RiskLevel::Low, None, "Shell")),
            _ => {}
        }
    }
    None
}

// -----------------------------------------------------------------------------
// 1. Power, Reboot, Shutdown & Session Controls
// -----------------------------------------------------------------------------
fn match_system_power_and_session(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    let is_systemd = ctx.init_system == InitSystem::Systemd;

    match lower {
        "reboot" | "restart pc" | "restart laptop" | "restart computer" | "restart system" => {
            let cmd = if is_systemd { "systemctl reboot" } else { "sudo reboot" };
            sugs.push(sug(cmd, "Reboot and restart the operating system", 0.99, RiskLevel::Medium, Some("Reboots system immediately".to_string()), "System"));
        }
        "shutdown" | "poweroff" | "power off" | "turn off" | "shut down" => {
            let cmd = if is_systemd { "systemctl poweroff" } else { "sudo poweroff" };
            sugs.push(sug(cmd, "Safely shut down and power off the machine", 0.99, RiskLevel::Destructive, Some("Powers off machine immediately".to_string()), "System"));
        }
        "sleep" | "suspend" => {
            let cmd = if is_systemd { "systemctl suspend" } else { "sudo zzz || sudo pm-suspend" };
            sugs.push(sug(cmd, "Suspend system into low-power RAM sleep mode", 0.99, RiskLevel::Low, None, "System"));
        }
        "hibernate" => {
            let cmd = if is_systemd { "systemctl hibernate" } else { "sudo ZZZ || sudo pm-hibernate" };
            sugs.push(sug(cmd, "Hibernate system state onto disk swap and power down", 0.98, RiskLevel::Medium, None, "System"));
        }
        "lock screen" | "lock" => {
            let cmd = match &ctx.display_server {
                DisplayServer::Wayland(wm) if wm == "hyprland" => "hyprlock",
                DisplayServer::Wayland(wm) if wm == "sway" => "swaylock",
                DisplayServer::Wayland(wm) if wm == "gnome" => "loginctl lock-session",
                DisplayServer::Wayland(wm) if wm == "kde" => "loginctl lock-session",
                DisplayServer::X11 => "xflock4 || i3lock || loginctl lock-session",
                _ => "loginctl lock-session",
            };
            sugs.push(sug(cmd, "Lock active desktop screen session", 0.99, RiskLevel::Low, None, "Desktop"));
        }
        "logout" | "exit desktop" | "quit desktop" => {
            let cmd = match &ctx.display_server {
                DisplayServer::Wayland(wm) if wm == "hyprland" => "hyprctl dispatch exit",
                DisplayServer::Wayland(wm) if wm == "sway" => "swaymsg exit",
                _ => "loginctl terminate-user $USER",
            };
            sugs.push(sug(cmd, "Exit desktop session and return to display manager", 0.98, RiskLevel::Medium, None, "Desktop"));
        }
        _ => return None,
    }
    Some(sugs)
}

// -----------------------------------------------------------------------------
// 2. Shell Typos & Quick Navigation
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
// 3. Standard Command Flags & Smart Parameter Completion
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
// 4. Developer Tools, Python venv, Rust, Docker & Shell Environment
// -----------------------------------------------------------------------------
fn match_developer_and_env(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    let is_fish = ctx.shell == "fish";
    let is_zsh = ctx.shell == "zsh";

    match lower {
        "reload shell" | "source shell" | "reload config" => {
            let cmd = if is_fish {
                "source ~/.config/fish/config.fish"
            } else if is_zsh {
                "source ~/.zshrc"
            } else {
                "source ~/.bashrc"
            };
            sugs.push(sug(cmd, "Reload current shell configuration live", 0.99, RiskLevel::Low, None, "Shell"));
        }
        "edit config" | "shell config" => {
            let cmd = if is_fish {
                "nano ~/.config/fish/config.fish"
            } else if is_zsh {
                "nano ~/.zshrc"
            } else {
                "nano ~/.bashrc"
            };
            sugs.push(sug(cmd, "Edit active shell profile configuration", 0.98, RiskLevel::Low, None, "Shell"));
        }
        "python venv" | "create venv" | "new venv" => {
            let act = if is_fish { "source .venv/bin/activate.fish" } else { "source .venv/bin/activate" };
            sugs.push(sug(&format!("python3 -m venv .venv && {}", act), "Create Python virtual environment in .venv and activate", 0.99, RiskLevel::Low, None, "Python"));
        }
        "clean rust" | "cargo clean" | "clean cargo" => {
            sugs.push(sug("cargo clean", "Remove target build directory and reclaim disk space", 0.99, RiskLevel::Low, None, "Rust"));
        }
        "update rust" | "rustup update" => {
            sugs.push(sug("rustup update", "Update Rust toolchain and components to latest version", 0.99, RiskLevel::Low, None, "Rust"));
        }
        "docker status" => {
            let cmd = if ctx.init_system == InitSystem::Systemd { "systemctl status docker" } else { "sudo rc-service docker status" };
            sugs.push(sug(cmd, "Inspect Docker daemon service operational status", 0.98, RiskLevel::Low, None, "Docker"));
        }
        "docker start" | "start docker" => {
            let cmd = if ctx.init_system == InitSystem::Systemd { "sudo systemctl start docker" } else { "sudo rc-service docker start" };
            sugs.push(sug(cmd, "Start the Docker container engine daemon", 0.98, RiskLevel::Medium, None, "Docker"));
        }
        "docker restart" | "restart docker" => {
            let cmd = if ctx.init_system == InitSystem::Systemd { "sudo systemctl restart docker" } else { "sudo rc-service docker restart" };
            sugs.push(sug(cmd, "Restart the Docker daemon engine", 0.98, RiskLevel::Medium, None, "Docker"));
        }
        "docker clean" | "clean docker" | "prune docker" => {
            sugs.push(sug("docker system prune -a --volumes", "Remove all stopped containers, unused networks, dangling images, and volumes", 0.98, RiskLevel::Destructive, Some("⚠ Removes all unused Docker containers and volumes".to_string()), "Docker"));
        }
        "generate ssh key" | "new ssh key" | "ssh key" => {
            sugs.push(sug("ssh-keygen -t ed25519 -C \"$USER@$(hostname)\"", "Generate modern, high-security Ed25519 SSH keypair", 0.98, RiskLevel::Low, None, "Security"));
        }
        "copy ssh key" | "show ssh key" => {
            let clip = match &ctx.display_server {
                DisplayServer::Wayland(_) => "wl-copy",
                DisplayServer::X11 => "xclip -selection clipboard",
                _ => "cat",
            };
            sugs.push(sug(&format!("cat ~/.ssh/id_ed25519.pub | {}", clip), "Copy public SSH key into clipboard buffer", 0.98, RiskLevel::Low, None, "Security"));
        }
        "kill steam" | "restart steam" => {
            sugs.push(sug("pkill -9 steam; steam &", "Force kill hung Steam client and restart in background", 0.98, RiskLevel::Medium, None, "Gaming"));
        }
        "gamemode status" | "check gamemode" => {
            sugs.push(sug("gamemoded -s", "Query Feral GameMode daemon active status", 0.98, RiskLevel::Low, None, "Gaming"));
        }
        _ => return None,
    }
    Some(sugs)
}

// -----------------------------------------------------------------------------
// 5. Path-Aware Project Execution & Building
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
            sugs.push(sug("python3 main.py", "Execute main Python entrypoint script", 0.98, RiskLevel::Low, None, "Project"));
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
// 7. Git Context Operations
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

    if lower == "check status" || lower == "check changes" || lower == "status" || lower == "check git" || lower == "git st" || lower == "git status" {
        sugs.push(sug("git status -s", "Show concise working tree status", 0.98, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git diff", "Inspect uncommitted code changes in working tree", 0.92, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "show diff" || lower == "diff" || lower == "check diff" {
        sugs.push(sug("git diff", "Show changes between working tree and index", 0.98, RiskLevel::Low, None, "Git"));
        sugs.push(sug("git diff --staged", "Show staged changes ready to be committed", 0.95, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    if lower == "push changes" || lower == "git push" || lower == "push" {
        sugs.push(sug("git push", "Push committed changes to remote repository", 0.98, RiskLevel::Low, None, "Git"));
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

    if lower == "commit history" || lower == "view commits" || lower == "git graph" {
        sugs.push(sug("git log --oneline --graph -n 15", "Display 15 most recent commits in compact branch tree", 0.98, RiskLevel::Low, None, "Git"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 8. Universal Package Management (pacman, apt, dnf, zypper, apk, xbps, etc.)
// -----------------------------------------------------------------------------
fn match_package_management(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    let pm = ctx.pkg_manager;

    // System Updates
    if lower == "update system" || lower == "upgrade system" || lower == "system update" || lower == "update" || lower == "upgrade" {
        sugs.push(sug(&pm.update_cmd(), &format!("Upgrade all system packages using {}", pm.name()), 0.99, RiskLevel::Low, None, "Packages"));
        if ctx.distro_family == DistroFamily::Arch && pm != crate::context::PackageManager::Paru {
            sugs.push(sug("paru -Syu", "Upgrade both official and AUR packages seamlessly", 0.94, RiskLevel::Low, None, "AUR"));
        }
        return Some(sugs);
    }

    // Clean cache
    if lower == "clean cache" || lower == "clean packages" || lower == "clear cache" {
        sugs.push(sug(&pm.clean_cache_cmd(), &format!("Clean cached package files using {}", pm.name()), 0.98, RiskLevel::Low, None, "Packages"));
        return Some(sugs);
    }

    // Clean orphans / unneeded dependencies
    if lower == "clean orphans" || lower == "remove orphans" || lower == "delete orphans" || lower == "autoremove" {
        let cmd = match ctx.distro_family {
            DistroFamily::Arch => "sudo pacman -Rns (pacman -Qtdq)",
            DistroFamily::Debian => "sudo apt autoremove",
            DistroFamily::Fedora => "sudo dnf autoremove",
            DistroFamily::OpenSuse => "sudo zypper packages --orphaned",
            DistroFamily::Void => "sudo xbps-remove -o",
            DistroFamily::Alpine => "sudo apk cache clean",
            _ => "sudo apt autoremove",
        };
        sugs.push(sug(cmd, "Remove unneeded orphaned dependencies", 0.98, RiskLevel::Destructive, Some("⚠ Removes unused package dependencies. Review list before confirming.".to_string()), "Packages"));
        return Some(sugs);
    }

    // Arch-specific lock removal & keyring repairs
    if ctx.distro_family == DistroFamily::Arch {
        if lower == "unlock pacman" || lower == "remove pacman lock" || lower == "pacman lock" || lower == "db.lck" {
            sugs.push(sug("sudo rm -f /var/lib/pacman/db.lck", "Remove stale pacman database lock file", 0.99, RiskLevel::Medium, None, "Pacman"));
            return Some(sugs);
        }
        if lower == "fix pacman keys" || lower == "fix keyring" || lower == "keyring error" || lower == "fix keys" {
            sugs.push(sug("sudo pacman -Sy archlinux-keyring cachyos-keyring && sudo pacman-key --refresh-keys", "Refresh and populate Arch Linux and CachyOS cryptographic GPG keyrings", 0.99, RiskLevel::Medium, None, "Pacman"));
            return Some(sugs);
        }
        if lower == "force update" || lower == "sync pacman" {
            sugs.push(sug("sudo pacman -Syyu", "Force refresh all package databases and run full upgrade", 0.99, RiskLevel::Low, None, "Pacman"));
            return Some(sugs);
        }
        if lower == "update mirrors" || lower == "fastest mirrors" {
            sugs.push(sug("sudo cachyos-rate-mirrors", "Benchmark and update CachyOS and Arch package mirrors", 0.98, RiskLevel::Low, None, "Pacman"));
            return Some(sugs);
        }
    }

    // Debian / Ubuntu specific PPA & fix broken
    if ctx.distro_family == DistroFamily::Debian {
        if lower == "fix broken" || lower == "fix apt" || lower == "fix dependencies" {
            sugs.push(sug("sudo apt --fix-broken install", "Repair broken dependency trees and missing packages", 0.99, RiskLevel::Medium, None, "Apt"));
            return Some(sugs);
        }
    }

    // List installed packages
    if lower == "list installed packages" || lower == "all packages" || lower == "list packages" {
        let cmd = match ctx.distro_family {
            DistroFamily::Arch => "pacman -Qe",
            DistroFamily::Debian => "apt list --installed",
            DistroFamily::Fedora => "dnf list installed",
            DistroFamily::OpenSuse => "zypper search -i",
            DistroFamily::Alpine => "apk info",
            DistroFamily::Void => "xbps-query -l",
            _ => "apt list --installed",
        };
        sugs.push(sug(cmd, "List all explicitly installed packages on the system", 0.98, RiskLevel::Low, None, "Packages"));
        return Some(sugs);
    }

    // Install intent
    if let Some(pkg) = parse_install_intent_str(lower) {
        if let Some((cmd, desc, cat)) = resolve_package_command(&pkg, ctx) {
            sugs.push(sug(&cmd, &desc, 0.98, RiskLevel::Low, None, &cat));
            return Some(sugs);
        }
    }

    // Search intent
    if let Some(pkg) = parse_find_package_intent_str(lower) {
        if is_valid_package_target(&pkg) {
            sugs.push(sug(&pm.search_cmd(&pkg), &format!("Search {} repositories for '{}'", pm.name(), pkg), 0.98, RiskLevel::Low, None, "Packages"));
            return Some(sugs);
        }
    }

    // Uninstall intent
    if let Some(pkg) = parse_remove_intent_str(lower) {
        if let Some((cmd, desc, cat)) = resolve_package_remove(&pkg, ctx) {
            sugs.push(sug(
                &cmd,
                &desc,
                0.98,
                RiskLevel::Destructive,
                Some("⚠ Removes package and related dependencies. Review list before confirming.".to_string()),
                &cat,
            ));
            return Some(sugs);
        }
    }

    None
}

// -----------------------------------------------------------------------------
// 9. Flatpak Management
// -----------------------------------------------------------------------------
fn match_flatpak_operations(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    match lower {
        "update flatpak" | "flatpak update" | "update flatpaks" => {
            sugs.push(sug("flatpak update", "Update all installed Flatpak applications and runtimes", 0.99, RiskLevel::Low, None, "Flatpak"));
        }
        "list flatpak" | "flatpak list" | "flatpaks" | "list flatpaks" => {
            sugs.push(sug("flatpak list --app", "List all installed Flatpak desktop applications", 0.99, RiskLevel::Low, None, "Flatpak"));
        }
        "clean flatpak" | "clean flatpaks" | "remove unused flatpak" => {
            sugs.push(sug("flatpak uninstall --unused", "Uninstall unused Flatpak runtimes to reclaim space", 0.98, RiskLevel::Low, None, "Flatpak"));
        }
        _ => {
            if let Some(caps) = Regex::new(r#"^(?:install flatpak|flatpak install)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?.captures(lower) {
                let app = &caps[1];
                sugs.push(sug(&format!("flatpak install flathub {}", app), &format!("Install Flatpak app '{}' from Flathub", app), 0.98, RiskLevel::Low, None, "Flatpak"));
            } else {
                return None;
            }
        }
    }
    Some(sugs)
}

// -----------------------------------------------------------------------------
// 10. Hardware, GPU (NVIDIA / AMD / Intel), Battery, Backlight & Audio
// -----------------------------------------------------------------------------
fn match_hardware_and_laptop(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // GPU commands (Vendor-aware)
    if lower == "check nvidia" || lower == "check gpu" || lower == "nvidia" || lower == "gpu info" || lower == "gpu temp" || lower == "gpu status" || matches_fuzzy(lower, "check nvidia", 0.78) {
        match ctx.gpu_vendor {
            GpuVendor::Nvidia => {
                sugs.push(sug("nvidia-smi", "Display NVIDIA GPU utilization, temperature, and VRAM allocation", 0.99, RiskLevel::Low, None, "Hardware"));
                sugs.push(sug("watch -n 1 nvidia-smi", "Monitor NVIDIA GPU stats live every second", 0.94, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Amd => {
                sugs.push(sug("radeontop", "Monitor AMD Radeon GPU utilization and VRAM in real-time", 0.99, RiskLevel::Low, None, "Hardware"));
                sugs.push(sug("rocm-smi", "Query AMD ROCm GPU clocks, temperature, and power metrics", 0.94, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Intel => {
                sugs.push(sug("sudo intel_gpu_top", "Monitor Intel Arc and integrated GPU engine render metrics", 0.99, RiskLevel::Low, None, "Hardware"));
            }
            GpuVendor::Generic => {
                sugs.push(sug("lspci -nnk | grep -A4 -E \"VGA|3D|Display\"", "Inspect graphics hardware and active kernel drivers", 0.98, RiskLevel::Low, None, "Hardware"));
            }
        }
        sugs.push(sug("lspci -nnk | grep -A4 -E \"VGA|3D|Display\"", "Inspect graphics controller and active kernel drivers", 0.88, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    if lower == "nvidia stats" || lower == "nvidia clocks" || lower == "gpu vram" {
        if ctx.gpu_vendor == GpuVendor::Nvidia {
            sugs.push(sug("nvidia-smi --query-gpu=utilization.gpu,utilization.memory,memory.used,memory.total,temperature.gpu,power.draw --format=csv -l 1", "Continuously print detailed GPU metrics in CSV format", 0.98, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
    }

    // Hybrid GPU switching / MUX Switch
    if lower == "gpu mode" || lower == "switch gpu" || lower == "mux switch" || lower == "hybrid gpu" {
        if ctx.has_asusctl {
            sugs.push(sug("supergfxctl -g", "Query current ASUS hybrid graphics mode", 0.98, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("supergfxctl -m Dedicated", "Switch to dedicated dGPU only (maximum performance)", 0.92, RiskLevel::Medium, None, "Hardware"));
            sugs.push(sug("supergfxctl -m Hybrid", "Switch to dynamic Intel/AMD + NVIDIA Hybrid mode", 0.92, RiskLevel::Medium, None, "Hardware"));
            sugs.push(sug("supergfxctl -m Integrated", "Switch to integrated graphics only (maximum battery life)", 0.92, RiskLevel::Medium, None, "Hardware"));
            return Some(sugs);
        } else {
            sugs.push(sug("prime-run <command>", "Launch command using dedicated high-performance GPU", 0.95, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("DRI_PRIME=1 <command>", "Run application on secondary discrete GPU via DRI PRIME", 0.92, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
    }

    // ASUS Keyboard Backlight & RGB
    if ctx.has_asusctl {
        if lower == "keyboard light" || lower == "keyboard brightness" || lower == "keyboard backlight" || lower == "aura" {
            sugs.push(sug("asusctl -k med", "Set ASUS TUF keyboard brightness to medium", 0.98, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("asusctl -k high", "Set ASUS TUF keyboard brightness to maximum", 0.95, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("asusctl -k off", "Turn off ASUS TUF keyboard backlight", 0.92, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
        if lower == "keyboard rgb" || lower == "rgb mode" || lower == "aura mode" {
            sugs.push(sug("asusctl led-mode static -c ffffff", "Set keyboard backlight to clean static white", 0.98, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("asusctl led-mode rainbow", "Set keyboard backlight to dynamic rainbow spectrum", 0.92, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
    }

    // Power & Fan profiles (Universal FreeDesktop standard + ASUS TUF)
    if lower == "fan turbo" || lower == "performance mode" || lower == "turbo fan" {
        if ctx.has_asusctl {
            sugs.push(sug("asusctl profile -P Performance", "Activate ASUS Performance/Turbo mode for maximum cooling & clocks", 0.98, RiskLevel::Low, None, "Hardware"));
        } else {
            sugs.push(sug("powerprofilesctl set performance", "Set Linux system-wide performance power profile", 0.98, RiskLevel::Low, None, "Hardware"));
        }
        return Some(sugs);
    }

    if lower == "fan quiet" || lower == "silent mode" || lower == "quiet mode" || lower == "power saver" {
        if ctx.has_asusctl {
            sugs.push(sug("asusctl profile -P Quiet", "Switch to Quiet/Silent power profile with reduced fan speeds", 0.98, RiskLevel::Low, None, "Hardware"));
        } else {
            sugs.push(sug("powerprofilesctl set power-saver", "Switch to power-saver profile to preserve battery", 0.98, RiskLevel::Low, None, "Hardware"));
        }
        return Some(sugs);
    }

    if lower == "fan balanced" || lower == "balanced mode" || lower == "normal fan" {
        if ctx.has_asusctl {
            sugs.push(sug("asusctl profile -P Balanced", "Switch to Balanced standard power and fan profile", 0.98, RiskLevel::Low, None, "Hardware"));
        } else {
            sugs.push(sug("powerprofilesctl set balanced", "Switch to balanced power profile", 0.98, RiskLevel::Low, None, "Hardware"));
        }
        return Some(sugs);
    }

    if lower == "check fan" || lower == "fan mode" || lower == "power profile" {
        if ctx.has_asusctl {
            sugs.push(sug("asusctl profile -p", "Display current active ASUS TUF fan and power profile", 0.98, RiskLevel::Low, None, "Hardware"));
        } else {
            sugs.push(sug("powerprofilesctl get", "Query current system power profile", 0.98, RiskLevel::Low, None, "Hardware"));
        }
        return Some(sugs);
    }

    // Laptop Battery & Brightness
    if ctx.form_factor == FormFactor::Laptop {
        if lower == "check battery" || lower == "battery status" || lower == "battery" {
            sugs.push(sug("upower -i /org/freedesktop/UPower/devices/battery_BAT0", "Show detailed battery health, percentage, and discharge rate", 0.98, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("cat /sys/class/power_supply/BAT*/capacity", "Print battery charge level percentage directly", 0.92, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
        if lower == "battery limit" || lower == "charge limit" || lower == "battery charge" {
            if ctx.has_asusctl {
                sugs.push(sug("asusctl -c 80", "Set battery charge limit to 80% to preserve battery lifespan", 0.98, RiskLevel::Low, None, "Hardware"));
                sugs.push(sug("asusctl -c 100", "Allow battery to charge fully to 100%", 0.92, RiskLevel::Low, None, "Hardware"));
                return Some(sugs);
            }
        }
        if lower == "brightness" || lower == "screen brightness" || lower == "brightness up" {
            sugs.push(sug("brightnessctl set +10%", "Increase display brightness by 10% using brightnessctl", 0.98, RiskLevel::Low, None, "Hardware"));
            sugs.push(sug("brightnessctl set 50%", "Set display brightness to 50%", 0.94, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
        if lower == "brightness down" || lower == "dim screen" {
            sugs.push(sug("brightnessctl set 10%-", "Decrease display brightness by 10% using brightnessctl", 0.98, RiskLevel::Low, None, "Hardware"));
            return Some(sugs);
        }
    }

    // Sensors & Temps
    if lower == "check sensors" || lower == "cpu temp" || lower == "temperatures" || lower == "sensors" {
        sugs.push(sug("sensors", "Display motherboard, CPU, and GPU temperature sensors", 0.98, RiskLevel::Low, None, "Hardware"));
        sugs.push(sug("watch sensors", "Monitor hardware thermal readings continuously", 0.92, RiskLevel::Low, None, "Hardware"));
        return Some(sugs);
    }

    // Universal Audio (PipeWire / PulseAudio / ALSA)
    if lower == "restart audio" || lower == "fix audio" || lower == "reload audio" || lower == "restart sound" || matches_fuzzy(lower, "restart audio", 0.78) {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "systemctl --user restart pipewire pipewire-pulse wireplumber",
            AudioSystem::Pulseaudio => "pulseaudio -k && pulseaudio --start",
            AudioSystem::Alsa => "sudo alsactl restore",
        };
        sugs.push(sug(cmd, "Restart audio sound server and session manager", 0.99, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "check audio" || lower == "audio status" || lower == "sound status" || lower == "audio devices" {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "wpctl status",
            AudioSystem::Pulseaudio => "pactl info",
            AudioSystem::Alsa => "alsamixer",
        };
        sugs.push(sug(cmd, "Inspect active audio sinks, sources, and volume levels", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "volume up" || lower == "increase volume" || lower == "louder" {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+",
            AudioSystem::Pulseaudio => "pactl set-sink-volume @DEFAULT_SINK@ +5%",
            AudioSystem::Alsa => "amixer set Master 5%+",
        };
        sugs.push(sug(cmd, "Increase system audio volume by 5%", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "volume down" || lower == "decrease volume" || lower == "quieter" {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-",
            AudioSystem::Pulseaudio => "pactl set-sink-volume @DEFAULT_SINK@ -5%",
            AudioSystem::Alsa => "amixer set Master 5%-",
        };
        sugs.push(sug(cmd, "Decrease system audio volume by 5%", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "mute mic" || lower == "mute microphone" {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle",
            AudioSystem::Pulseaudio => "pactl set-source-mute @DEFAULT_SOURCE@ toggle",
            AudioSystem::Alsa => "amixer set Capture toggle",
        };
        sugs.push(sug(cmd, "Toggle microphone mute on default audio input source", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "mute audio" || lower == "mute sound" || lower == "mute volume" || lower == "mute" {
        let cmd = match ctx.audio_system {
            AudioSystem::Pipewire => "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle",
            AudioSystem::Pulseaudio => "pactl set-sink-mute @DEFAULT_SINK@ toggle",
            AudioSystem::Alsa => "amixer set Master toggle",
        };
        sugs.push(sug(cmd, "Toggle audio playback mute on default output sink", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    if lower == "pavucontrol" || lower == "sound mixer" || lower == "volume control" {
        sugs.push(sug("pavucontrol", "Open PulseAudio / PipeWire graphical volume control mixer", 0.98, RiskLevel::Low, None, "Audio"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 11. Services & System Administration (systemd / OpenRC / runit aware)
// -----------------------------------------------------------------------------
fn match_services_and_init(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();
    let is_systemd = ctx.init_system == InitSystem::Systemd;

    // Bluetooth
    if lower == "restart bluetooth" || matches_fuzzy(lower, "restart bluetooth", 0.8) {
        let cmd = if is_systemd { "sudo systemctl restart bluetooth" } else { "sudo rc-service bluetooth restart" };
        sugs.push(sug(cmd, "Restart the Bluetooth hardware daemon", 0.98, RiskLevel::Medium, None, "System"));
        return Some(sugs);
    }
    if lower == "check bluetooth" || lower == "bluetooth status" || matches_fuzzy(lower, "status bluetooth", 0.8) {
        let cmd = if is_systemd { "systemctl status bluetooth" } else { "sudo rc-service bluetooth status" };
        sugs.push(sug(cmd, "Inspect Bluetooth service operational status", 0.98, RiskLevel::Low, None, "System"));
        return Some(sugs);
    }

    // Failed services
    if lower == "failed services" || lower == "check failed" || lower == "systemctl failed" {
        if is_systemd {
            sugs.push(sug("systemctl --failed", "List all systemd system services that failed to start", 0.99, RiskLevel::Low, None, "System"));
            sugs.push(sug("systemctl --user --failed", "List failed user session services", 0.94, RiskLevel::Low, None, "System"));
            return Some(sugs);
        }
    }

    // System logs
    if lower == "system logs" || lower == "check logs" || lower == "error logs" || lower == "journal" {
        if is_systemd {
            sugs.push(sug("journalctl -xe", "Inspect latest system log entries with explanatory error catalog", 0.98, RiskLevel::Low, None, "System"));
            return Some(sugs);
        } else {
            sugs.push(sug("tail -n 50 /var/log/messages", "Inspect recent system log entries", 0.98, RiskLevel::Low, None, "System"));
            return Some(sugs);
        }
    }
    if lower == "boot logs" || lower == "check boot" {
        if is_systemd {
            sugs.push(sug("journalctl -b", "View journal logs recorded during the current system boot", 0.98, RiskLevel::Low, None, "System"));
            return Some(sugs);
        }
    }
    if lower == "kernel logs" || lower == "dmesg" {
        sugs.push(sug("sudo dmesg -T | tail -n 50", "Inspect 50 most recent human-readable Linux kernel ring buffer logs", 0.98, RiskLevel::Low, None, "System"));
        return Some(sugs);
    }

    // Pattern: restart/start/stop/status <service>
    let service_regex = Regex::new(r#"^(start|restart|stop|status|enable|disable)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?;
    if let Some(caps) = service_regex.captures(lower) {
        let action = &caps[1];
        let service = &caps[2];

        if is_systemd {
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
            sugs.push(sug(&cmd, &format!("Execute systemd '{}' on service '{}'", action, service), 0.97, risk, None, "System"));
            return Some(sugs);
        } else if ctx.init_system == InitSystem::OpenRc {
            let cmd = format!("sudo rc-service {} {}", service, action);
            sugs.push(sug(&cmd, &format!("Execute OpenRC '{}' on service '{}'", action, service), 0.97, RiskLevel::Medium, None, "System"));
            return Some(sugs);
        }
    }

    None
}

// -----------------------------------------------------------------------------
// 12. Network, Ports, IP & Connectivity
// -----------------------------------------------------------------------------
fn match_network_and_ports(lower: &str) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // Kill port
    if let Some(caps) = Regex::new(r#"^(?:kill port|stop port|free port)\s+(\d{1,5})$"#).ok()?.captures(lower) {
        let port = &caps[1];
        sugs.push(sug(&format!("fuser -k {}/tcp", port), &format!("Terminate process bound to TCP port {}", port), 0.98, RiskLevel::Medium, Some(format!("Kills process listening on port {}", port)), "Network"));
        return Some(sugs);
    }

    // Port lookup
    if let Some(port) = parse_port_intent_str(lower) {
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
    if lower == "test internet" || lower == "ping test" || lower == "check internet" || lower == "ping" {
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
// 13. Files, Storage, Disks, Search & Permissions
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
// 14. Process Management, Performance & Memory
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
        if proc.len() >= 3 || ["dd", "cp", "mv", "ps", "sh"].contains(&proc) {
            sugs.push(sug(&format!("pkill -f \"{}\"", proc), &format!("Terminate all active processes matching '{}'", proc), 0.97, RiskLevel::Medium, Some("Terminates matching process".to_string()), "Processes"));
            return Some(sugs);
        }
    }

    if let Some(caps) = Regex::new(r#"^(?:find process|check process|pgrep)\s+([a-zA-Z0-9_\-\.]+)$"#).ok()?.captures(lower) {
        let proc = &caps[1];
        if proc.len() >= 3 || ["dd", "cp", "mv", "ps", "sh"].contains(&proc) {
            sugs.push(sug(&format!("pgrep -fl \"{}\"", proc), &format!("List process IDs and command lines matching '{}'", proc), 0.98, RiskLevel::Low, None, "Processes"));
            return Some(sugs);
        }
    }

    None
}

// -----------------------------------------------------------------------------
// 15. Desktop Environment & Window Manager (Hyprland, Sway, GNOME, KDE, X11)
// -----------------------------------------------------------------------------
fn match_desktop_and_window_manager(lower: &str, ctx: &SystemContext) -> Option<Vec<CandidateSuggestion>> {
    let mut sugs = Vec::new();

    // Reload compositor / window manager
    if lower == "reload hyprland" || lower == "reload desktop" || lower == "reload wm" || lower == "hyprland reload" {
        match &ctx.display_server {
            DisplayServer::Wayland(wm) if wm == "hyprland" => {
                sugs.push(sug("hyprctl reload", "Reload Hyprland compositor configuration live without exiting", 0.99, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::Wayland(wm) if wm == "sway" => {
                sugs.push(sug("swaymsg reload", "Reload Sway compositor configuration live", 0.99, RiskLevel::Low, None, "Desktop"));
            }
            _ => {
                sugs.push(sug("hyprctl reload || swaymsg reload", "Reload active Wayland compositor configuration", 0.95, RiskLevel::Low, None, "Desktop"));
            }
        }
        return Some(sugs);
    }

    // List windows
    if lower == "list windows" || lower == "show windows" || lower == "open windows" || lower == "hyprland clients" {
        match &ctx.display_server {
            DisplayServer::Wayland(wm) if wm == "hyprland" => {
                sugs.push(sug("hyprctl clients", "List all open Wayland windows, workspaces, and classes in Hyprland", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::Wayland(wm) if wm == "sway" => {
                sugs.push(sug("swaymsg -t get_tree", "List all open Sway desktop windows and containers", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::X11 => {
                sugs.push(sug("wmctrl -l", "List all active X11 windows", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            _ => {
                sugs.push(sug("hyprctl clients", "List open windows and workspaces", 0.90, RiskLevel::Low, None, "Desktop"));
            }
        }
        return Some(sugs);
    }

    // Active window
    if lower == "active window" || lower == "current window" {
        if let DisplayServer::Wayland(wm) = &ctx.display_server {
            if wm == "hyprland" {
                sugs.push(sug("hyprctl activewindow", "Inspect title, PID, and address of currently focused window", 0.98, RiskLevel::Low, None, "Desktop"));
                return Some(sugs);
            }
        }
        sugs.push(sug("hyprctl activewindow || xdotool getwindowfocus getwindowname", "Display active window details", 0.92, RiskLevel::Low, None, "Desktop"));
        return Some(sugs);
    }

    // Kill window
    if lower == "kill window" || lower == "force quit" || lower == "force kill" {
        if let DisplayServer::Wayland(wm) = &ctx.display_server {
            if wm == "hyprland" {
                sugs.push(sug("hyprctl kill", "Click on any Wayland window to forcefully kill its process", 0.98, RiskLevel::Medium, None, "Desktop"));
                return Some(sugs);
            }
        }
        sugs.push(sug("xkill || hyprctl kill", "Force kill target window by clicking on it", 0.95, RiskLevel::Medium, None, "Desktop"));
        return Some(sugs);
    }

    // Monitors / Displays
    if lower == "list monitors" || lower == "check monitors" || lower == "displays" || lower == "monitors" {
        match &ctx.display_server {
            DisplayServer::Wayland(wm) if wm == "hyprland" => {
                sugs.push(sug("hyprctl monitors", "Display connected monitors, active resolutions, scaling, and refresh rates", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::Wayland(wm) if wm == "sway" => {
                sugs.push(sug("swaymsg -t get_outputs", "Display connected Sway display outputs", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::X11 => {
                sugs.push(sug("xrandr --query", "Query connected displays and supported resolutions via xrandr", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            _ => {
                sugs.push(sug("hyprctl monitors || xrandr --query", "Inspect connected displays and screen resolutions", 0.92, RiskLevel::Low, None, "Desktop"));
            }
        }
        return Some(sugs);
    }

    // Screenshots
    if lower == "screenshot" || lower == "take screenshot" || lower == "screen capture" {
        match &ctx.display_server {
            DisplayServer::Wayland(_) => {
                sugs.push(sug("grim -g \"$(slurp)\" ~/Pictures/screenshot_(date +%Y%m%d_%H%M%S).png", "Select a screen region and save screenshot to Pictures folder", 0.98, RiskLevel::Low, None, "Desktop"));
                sugs.push(sug("hyprshot -m region", "Interactive Wayland region screenshot tool", 0.93, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::X11 => {
                sugs.push(sug("scrot -s ~/Pictures/screenshot_%Y%m%d_%H%M%S.png", "Select a screen region and save screenshot via scrot", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            _ => {
                sugs.push(sug("grim -g \"$(slurp)\" ~/Pictures/screenshot_(date +%Y%m%d_%H%M%S).png", "Take region screenshot", 0.90, RiskLevel::Low, None, "Desktop"));
            }
        }
        return Some(sugs);
    }

    // Screen recording
    if lower == "screen record" || lower == "record screen" || lower == "record desktop" {
        if matches!(ctx.display_server, DisplayServer::Wayland(_)) {
            sugs.push(sug("wf-recorder -g \"$(slurp)\" -f ~/Videos/recording_(date +%Y%m%d_%H%M%S).mp4", "Select region and record desktop video using wf-recorder", 0.98, RiskLevel::Low, None, "Desktop"));
            return Some(sugs);
        }
    }

    // Clipboard history
    if lower == "clipboard history" || lower == "cliphist" || lower == "clipboard" {
        match &ctx.display_server {
            DisplayServer::Wayland(_) => {
                sugs.push(sug("cliphist list | rofi -dmenu | cliphist decode | wl-copy", "Search clipboard history with rofi and copy selection to clipboard", 0.98, RiskLevel::Low, None, "Desktop"));
                sugs.push(sug("wl-paste", "Output current Wayland clipboard text to stdout", 0.92, RiskLevel::Low, None, "Desktop"));
            }
            DisplayServer::X11 => {
                sugs.push(sug("xclip -selection clipboard -o", "Print current X11 clipboard contents to stdout", 0.98, RiskLevel::Low, None, "Desktop"));
            }
            _ => {
                sugs.push(sug("wl-paste || xclip -selection clipboard -o", "Read clipboard contents", 0.90, RiskLevel::Low, None, "Desktop"));
            }
        }
        return Some(sugs);
    }

    if lower == "paste clipboard" || lower == "paste" {
        let cmd = match &ctx.display_server {
            DisplayServer::Wayland(_) => "wl-paste",
            DisplayServer::X11 => "xclip -selection clipboard -o",
            _ => "wl-paste",
        };
        sugs.push(sug(cmd, "Output current clipboard text to stdout", 0.98, RiskLevel::Low, None, "Desktop"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// 16. Distro-Specific Extras (Btrfs, Snapper, CachyOS)
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

    if lower == "btrfs balance" || lower == "balance btrfs" {
        sugs.push(sug("sudo btrfs balance start -dusage=50 -musage=50 /", "Rebalance and compact underallocated Btrfs block groups", 0.98, RiskLevel::Medium, None, "Btrfs"));
        return Some(sugs);
    }

    if lower == "cachyos hello" {
        sugs.push(sug("cachyos-hello", "Open CachyOS Welcome assistant application", 0.98, RiskLevel::Low, None, "CachyOS"));
        return Some(sugs);
    }
    if lower == "cachyos kernel" || lower == "kernel manager" {
        sugs.push(sug("cachyos-kernel-manager", "Open CachyOS graphical kernel manager utility", 0.98, RiskLevel::Low, None, "CachyOS"));
        return Some(sugs);
    }
    if lower == "cachyos package installer" || lower == "cachyos software" {
        sugs.push(sug("cachyos-packageinstaller", "Launch graphical CachyOS package installer", 0.98, RiskLevel::Low, None, "CachyOS"));
        return Some(sugs);
    }

    None
}

// -----------------------------------------------------------------------------
// Helper parsers & universal lookups
// -----------------------------------------------------------------------------
const KNOWN_SHORT_PACKAGES: &[&str] = &[
    "go", "r", "jq", "gh", "fd", "rg", "du", "ip", "bt", "pv", "bc", "nc", "xz", "7z", "cp", "mv", "dd", "vi", "ps", "sh",
];

pub fn is_valid_package_target(pkg: &str) -> bool {
    let trimmed = pkg.trim().to_lowercase();
    if trimmed.is_empty() {
        return false;
    }
    // Filter out common stopwords, noise words, and articles
    if [
        "it", "this", "that", "them", "all", "app", "application", "tool",
        "pkg", "package", "packages", "repo", "for", "via", "aur",
    ].contains(&trimmed.as_str()) {
        return false;
    }
    // Allow known valid 1-2 letter Linux packages
    if KNOWN_SHORT_PACKAGES.contains(&trimmed.as_str()) {
        return true;
    }
    // Any package name not in the short list MUST have at least 3 characters
    if trimmed.len() < 3 {
        return false;
    }
    // Must be valid package identifier characters (spaces allowed for multi-word names like "google chrome")
    trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '+' || c == ' ')
}

fn parse_install_intent(tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }

    let install_idx = tokens.iter().position(|t| {
        classify_action_token(t) == Some(ActionVerb::Install)
    })?;

    let pkg_tokens: Vec<&str> = tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != install_idx)
        .map(|(_, t)| t.as_str())
        .filter(|t| !["aur", "package", "packages", "app", "application", "tool", "via"].contains(t))
        .collect();

    if pkg_tokens.is_empty() {
        return None;
    }

    let joined = pkg_tokens.join(" ");
    if !is_valid_package_target(&joined) {
        return None;
    }

    Some(joined)
}

fn parse_install_intent_str(lower: &str) -> Option<String> {
    let (_, tokens) = clean_intent(lower);
    parse_install_intent(&tokens)
}

fn parse_remove_intent(tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }

    let remove_idx = tokens.iter().position(|t| {
        classify_action_token(t) == Some(ActionVerb::Remove)
    })?;

    let pkg_tokens: Vec<&str> = tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != remove_idx)
        .map(|(_, t)| t.as_str())
        .filter(|t| !["package", "packages", "app", "application", "tool", "orphans", "orphan", "cache"].contains(t))
        .collect();

    if pkg_tokens.is_empty() {
        return None;
    }

    let joined = pkg_tokens.join(" ");
    if !is_valid_package_target(&joined) {
        return None;
    }

    Some(joined)
}

#[allow(dead_code)]
fn parse_remove_intent_str(lower: &str) -> Option<String> {
    let (_, tokens) = clean_intent(lower);
    parse_remove_intent(&tokens)
}

fn parse_find_package_intent(tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }

    let find_idx = tokens.iter().position(|t| {
        let l = t.as_str();
        l == "search" || l == "find" || l == "lookup" || l == "locate"
    })?;

    let pkg_tokens: Vec<&str> = tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != find_idx)
        .map(|(_, t)| t.as_str())
        .filter(|t| !["package", "packages", "app", "repo", "for"].contains(t))
        .collect();

    if pkg_tokens.is_empty() {
        return None;
    }

    let joined = pkg_tokens.join(" ");
    if !is_valid_package_target(&joined) {
        return None;
    }

    Some(joined)
}

fn parse_find_package_intent_str(lower: &str) -> Option<String> {
    let (_, tokens) = clean_intent(lower);
    parse_find_package_intent(&tokens)
}

fn parse_port_intent(cleaned: &str, tokens: &[String]) -> Option<(String, bool)> {
    let has_port_hint = cleaned.contains("port") || cleaned.contains("prt") || cleaned.contains("listen")
        || cleaned.contains("kill") || cleaned.contains("using") || cleaned.contains("stop");

    let re = Regex::new(r#"\b([1-9]\d{1,4})\b"#).ok()?;
    let caps = re.captures(cleaned)?;
    let port_str = caps.get(1)?.as_str();
    let port_num = port_str.parse::<u32>().ok()?;

    if port_num > 0 && port_num <= 65535 {
        if has_port_hint || tokens.len() <= 2 {
            let is_kill = tokens.iter().any(|t| {
                matches!(classify_action_token(t), Some(ActionVerb::Kill | ActionVerb::Remove))
            });
            return Some((port_str.to_string(), is_kill));
        }
    }
    None
}

fn parse_port_intent_str(lower: &str) -> Option<String> {
    let (cleaned, tokens) = clean_intent(lower);
    parse_port_intent(&cleaned, &tokens).map(|(p, _)| p)
}

// Universal package cross-distro mapping
struct UniversalPackage {
    key: &'static str,
    arch_pkg: &'static str,
    debian_pkg: &'static str,
    fedora_pkg: &'static str,
    desc: &'static str,
    is_aur: bool,
}

const UNIVERSAL_PACKAGES: &[UniversalPackage] = &[
    // Browsers
    UniversalPackage { key: "chrome", arch_pkg: "google-chrome", debian_pkg: "google-chrome-stable", fedora_pkg: "google-chrome-stable", desc: "Google Chrome web browser", is_aur: true },
    UniversalPackage { key: "google chrome", arch_pkg: "google-chrome", debian_pkg: "google-chrome-stable", fedora_pkg: "google-chrome-stable", desc: "Google Chrome web browser", is_aur: true },
    UniversalPackage { key: "chorme", arch_pkg: "google-chrome", debian_pkg: "google-chrome-stable", fedora_pkg: "google-chrome-stable", desc: "Google Chrome web browser", is_aur: true },
    UniversalPackage { key: "chromium", arch_pkg: "chromium", debian_pkg: "chromium-browser", fedora_pkg: "chromium", desc: "Chromium open-source web browser", is_aur: false },
    UniversalPackage { key: "firefox", arch_pkg: "firefox", debian_pkg: "firefox", fedora_pkg: "firefox", desc: "Mozilla Firefox web browser", is_aur: false },
    UniversalPackage { key: "firfox", arch_pkg: "firefox", debian_pkg: "firefox", fedora_pkg: "firefox", desc: "Mozilla Firefox web browser", is_aur: false },
    UniversalPackage { key: "brave", arch_pkg: "brave-bin", debian_pkg: "brave-browser", fedora_pkg: "brave-browser", desc: "Brave privacy web browser", is_aur: true },
    UniversalPackage { key: "brave browser", arch_pkg: "brave-bin", debian_pkg: "brave-browser", fedora_pkg: "brave-browser", desc: "Brave privacy web browser", is_aur: true },
    UniversalPackage { key: "zen", arch_pkg: "zen-browser-bin", debian_pkg: "flatpak install flathub io.github.zen_browser.zen", fedora_pkg: "flatpak install flathub io.github.zen_browser.zen", desc: "Zen modern privacy browser", is_aur: true },

    // Development & IDEs
    UniversalPackage { key: "vscode", arch_pkg: "code", debian_pkg: "code", fedora_pkg: "code", desc: "Visual Studio Code editor", is_aur: false },
    UniversalPackage { key: "code", arch_pkg: "code", debian_pkg: "code", fedora_pkg: "code", desc: "Visual Studio Code editor", is_aur: false },
    UniversalPackage { key: "vscod", arch_pkg: "code", debian_pkg: "code", fedora_pkg: "code", desc: "Visual Studio Code editor", is_aur: false },
    UniversalPackage { key: "visual studio code", arch_pkg: "visual-studio-code-bin", debian_pkg: "code", fedora_pkg: "code", desc: "Visual Studio Code proprietary binary", is_aur: true },
    UniversalPackage { key: "vscodium", arch_pkg: "vscodium-bin", debian_pkg: "codium", fedora_pkg: "codium", desc: "VSCodium telemetry-free VS Code", is_aur: true },
    UniversalPackage { key: "zed", arch_pkg: "zed", debian_pkg: "flatpak install flathub dev.zed.Zed", fedora_pkg: "zed", desc: "Zed high-performance code editor", is_aur: false },
    UniversalPackage { key: "neovim", arch_pkg: "neovim", debian_pkg: "neovim", fedora_pkg: "neovim", desc: "Vim-fork focused on extensibility", is_aur: false },
    UniversalPackage { key: "nvim", arch_pkg: "neovim", debian_pkg: "neovim", fedora_pkg: "neovim", desc: "Vim-fork text editor", is_aur: false },
    UniversalPackage { key: "emacs", arch_pkg: "emacs", debian_pkg: "emacs", fedora_pkg: "emacs", desc: "Extensible GNU text editor", is_aur: false },
    UniversalPackage { key: "build essential", arch_pkg: "base-devel", debian_pkg: "build-essential", fedora_pkg: "groupinstall \"Development Tools\"", desc: "Base compilation toolchain and compilers", is_aur: false },
    UniversalPackage { key: "docker", arch_pkg: "docker docker-compose", debian_pkg: "docker.io docker-compose", fedora_pkg: "docker docker-compose", desc: "Docker containerization engine and compose", is_aur: false },

    // Languages
    UniversalPackage { key: "rust", arch_pkg: "rustup", debian_pkg: "rustup || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh", fedora_pkg: "rustup", desc: "Rust toolchain installer and compiler", is_aur: false },
    UniversalPackage { key: "node", arch_pkg: "nodejs npm", debian_pkg: "nodejs npm", fedora_pkg: "nodejs npm", desc: "Node.js JavaScript runtime and npm", is_aur: false },
    UniversalPackage { key: "python", arch_pkg: "python python-pip", debian_pkg: "python3 python3-pip python3-venv", fedora_pkg: "python3 python3-pip", desc: "Python programming language and pip", is_aur: false },
    UniversalPackage { key: "go", arch_pkg: "go", debian_pkg: "golang-go", fedora_pkg: "golang", desc: "Go programming language compiler", is_aur: false },

    // CLI Tools
    UniversalPackage { key: "git", arch_pkg: "git", debian_pkg: "git", fedora_pkg: "git", desc: "Git version control system", is_aur: false },
    UniversalPackage { key: "curl", arch_pkg: "curl", debian_pkg: "curl", fedora_pkg: "curl", desc: "Command line data transfer tool", is_aur: false },
    UniversalPackage { key: "ripgrep", arch_pkg: "ripgrep", debian_pkg: "ripgrep", fedora_pkg: "ripgrep", desc: "Fast line-oriented search tool", is_aur: false },
    UniversalPackage { key: "fd", arch_pkg: "fd", debian_pkg: "fd-find", fedora_pkg: "fd-find", desc: "Simple and fast alternative to find", is_aur: false },
    UniversalPackage { key: "bat", arch_pkg: "bat", debian_pkg: "bat", fedora_pkg: "bat", desc: "Cat clone with syntax highlighting", is_aur: false },
    UniversalPackage { key: "eza", arch_pkg: "eza", debian_pkg: "eza", fedora_pkg: "eza", desc: "Modern, maintained replacement for ls", is_aur: false },
    UniversalPackage { key: "yazi", arch_pkg: "yazi", debian_pkg: "cargo install yazi-fm yazi-cli", fedora_pkg: "yazi", desc: "Blazing-fast terminal file manager", is_aur: false },
    UniversalPackage { key: "btop", arch_pkg: "btop", debian_pkg: "btop", fedora_pkg: "btop", desc: "Resource monitor that shows usage and stats", is_aur: false },
    UniversalPackage { key: "htop", arch_pkg: "htop", debian_pkg: "htop", fedora_pkg: "htop", desc: "Interactive process viewer", is_aur: false },
    UniversalPackage { key: "fastfetch", arch_pkg: "fastfetch", debian_pkg: "fastfetch", fedora_pkg: "fastfetch", desc: "Fast system information display tool", is_aur: false },
    UniversalPackage { key: "tmux", arch_pkg: "tmux", debian_pkg: "tmux", fedora_pkg: "tmux", desc: "Terminal multiplexer", is_aur: false },
    UniversalPackage { key: "fzf", arch_pkg: "fzf", debian_pkg: "fzf", fedora_pkg: "fzf", desc: "Command-line fuzzy finder", is_aur: false },

    // Gaming & Media
    UniversalPackage { key: "steam", arch_pkg: "steam", debian_pkg: "steam", fedora_pkg: "steam", desc: "Valve Steam gaming platform", is_aur: false },
    UniversalPackage { key: "lutris", arch_pkg: "lutris", debian_pkg: "lutris", fedora_pkg: "lutris", desc: "Open gaming platform for Linux", is_aur: false },
    UniversalPackage { key: "spotify", arch_pkg: "spotify", debian_pkg: "flatpak install flathub com.spotify.Client", fedora_pkg: "flatpak install flathub com.spotify.Client", desc: "Spotify music streaming client", is_aur: true },
    UniversalPackage { key: "discord", arch_pkg: "discord", debian_pkg: "discord", fedora_pkg: "discord", desc: "Discord voice and text communication client", is_aur: false },
    UniversalPackage { key: "dicsord", arch_pkg: "discord", debian_pkg: "discord", fedora_pkg: "discord", desc: "Discord voice and text communication client", is_aur: false },
    UniversalPackage { key: "vlc", arch_pkg: "vlc", debian_pkg: "vlc", fedora_pkg: "vlc", desc: "VLC multimedia player framework", is_aur: false },
    UniversalPackage { key: "mpv", arch_pkg: "mpv", debian_pkg: "mpv", fedora_pkg: "mpv", desc: "Lightweight command-line video player", is_aur: false },
    UniversalPackage { key: "obs", arch_pkg: "obs-studio", debian_pkg: "obs-studio", fedora_pkg: "obs-studio", desc: "OBS Studio recording and streaming", is_aur: false },
    UniversalPackage { key: "blender", arch_pkg: "blender", debian_pkg: "blender", fedora_pkg: "blender", desc: "Blender 3D computer graphics suite", is_aur: false },
    UniversalPackage { key: "gimp", arch_pkg: "gimp", debian_pkg: "gimp", fedora_pkg: "gimp", desc: "GNU Image Manipulation Program", is_aur: false },
    UniversalPackage { key: "ffmpeg", arch_pkg: "ffmpeg", debian_pkg: "ffmpeg", fedora_pkg: "ffmpeg", desc: "Complete media recorder and converter", is_aur: false },

    // Hardware & Utilities
    UniversalPackage { key: "supergfxctl", arch_pkg: "supergfxctl", debian_pkg: "supergfxctl", fedora_pkg: "supergfxctl", desc: "ASUS hybrid graphics mode switcher", is_aur: false },
    UniversalPackage { key: "asusctl", arch_pkg: "asusctl", debian_pkg: "asusctl", fedora_pkg: "asusctl", desc: "ASUS ROG/TUF laptop control utility", is_aur: false },
    UniversalPackage { key: "ghostty", arch_pkg: "ghostty", debian_pkg: "flatpak install flathub com.mitchellh.ghostty", fedora_pkg: "ghostty", desc: "Fast native GPU terminal emulator", is_aur: true },
    UniversalPackage { key: "kitty", arch_pkg: "kitty", debian_pkg: "kitty", fedora_pkg: "kitty", desc: "GPU-accelerated terminal emulator", is_aur: false },
    UniversalPackage { key: "alacritty", arch_pkg: "alacritty", debian_pkg: "alacritty", fedora_pkg: "alacritty", desc: "Cross-platform GPU terminal emulator", is_aur: false },
];

/// Resolve a package install command. Returns None for inputs that are too short/ambiguous.
fn resolve_package_command(pkg: &str, ctx: &SystemContext) -> Option<(String, String, String)> {
    let lower_pkg = pkg.to_lowercase();
    let lower_pkg = lower_pkg.trim();

    // Reject obviously incomplete / too-short inputs that aren't known short packages
    if !is_valid_package_target(lower_pkg) {
        return None;
    }

    let pm = ctx.pkg_manager;

    // 1. Exact match in catalog
    for entry in UNIVERSAL_PACKAGES {
        if lower_pkg == entry.key || lower_pkg == entry.arch_pkg {
            return Some(format_package_install(entry, ctx));
        }
    }

    // 2. The user typed something that CONTAINS a catalog key as a full word
    //    e.g. "google chrome" contains "chrome", "brave browser" contains "brave"
    //    NEVER match the reverse (entry.key contains lower_pkg) — that causes "l" to match
    //    every entry that has "l" in its key.
    for entry in UNIVERSAL_PACKAGES {
        if lower_pkg.contains(entry.key) && entry.key.len() >= 3 {
            return Some(format_package_install(entry, ctx));
        }
    }

    // 3. Prefix match: user typed a prefix of a catalog key (min 4 chars)
    if lower_pkg.len() >= 4 {
        for entry in UNIVERSAL_PACKAGES {
            if entry.key.starts_with(lower_pkg) {
                return Some(format_package_install(entry, ctx));
            }
        }
    }

    // 4. Fuzzy match entire string in catalog (high threshold)
    let keys: Vec<&str> = UNIVERSAL_PACKAGES.iter().map(|p| p.key).collect();
    if let Some((best_key, score)) = fuzzy_find_best(lower_pkg, &keys, 0.72) {
        if let Some(entry) = UNIVERSAL_PACKAGES.iter().find(|p| p.key == best_key) {
            let (cmd, desc, cat) = format_package_install(entry, ctx);
            return Some((cmd, format!("{} (fuzzy match {:.0}%)", desc, score * 100.0), cat));
        }
    }

    // 5. Fuzzy match individual words (only for words ≥ 4 chars to avoid false positives)
    for word in lower_pkg.split_whitespace() {
        if word.len() >= 4 {
            if let Some((best_key, score)) = fuzzy_find_best(word, &keys, 0.75) {
                if let Some(entry) = UNIVERSAL_PACKAGES.iter().find(|p| p.key == best_key) {
                    let (cmd, desc, cat) = format_package_install(entry, ctx);
                    return Some((cmd, format!("{} (fuzzy match {:.0}%)", desc, score * 100.0), cat));
                }
            }
        }
    }

    // 6. Only fall back to native package manager for well-formed package names (≥ 3 chars)
    if lower_pkg.len() >= 3 {
        return Some((
            pm.install_cmd(lower_pkg),
            format!("Install '{}' via {}", lower_pkg, pm.name()),
            pm.name().to_string(),
        ));
    }

    None
}

/// Resolve a package remove/uninstall command. Returns None for incomplete/ambiguous inputs.
fn resolve_package_remove(pkg: &str, ctx: &SystemContext) -> Option<(String, String, String)> {
    let lower_pkg = pkg.to_lowercase();
    let lower_pkg = lower_pkg.trim();

    // Reject short/incomplete package names
    if !is_valid_package_target(lower_pkg) {
        return None;
    }

    let pm = ctx.pkg_manager;

    // 1. Exact catalog key match → use the distro-specific package name
    for entry in UNIVERSAL_PACKAGES {
        if lower_pkg == entry.key {
            let arch_name = entry.arch_pkg;
            return Some((
                pm.remove_cmd(arch_name),
                format!("Uninstall {} ({})", entry.desc, pm.name()),
                pm.name().to_string(),
            ));
        }
    }

    // 2. The user typed a phrase containing a catalog key (e.g. "google chrome")
    for entry in UNIVERSAL_PACKAGES {
        if lower_pkg.contains(entry.key) && entry.key.len() >= 3 {
            let arch_name = entry.arch_pkg;
            return Some((
                pm.remove_cmd(arch_name),
                format!("Uninstall {} ({})", entry.desc, pm.name()),
                pm.name().to_string(),
            ));
        }
    }

    // 3. Fuzzy match (high threshold to avoid false positives on destructive ops)
    let keys: Vec<&str> = UNIVERSAL_PACKAGES.iter().map(|p| p.key).collect();
    if let Some((best_key, score)) = fuzzy_find_best(lower_pkg, &keys, 0.78) {
        if let Some(entry) = UNIVERSAL_PACKAGES.iter().find(|p| p.key == best_key) {
            let arch_name = entry.arch_pkg;
            return Some((
                pm.remove_cmd(arch_name),
                format!("Uninstall {} ({:.0}% match, {})", entry.desc, score * 100.0, pm.name()),
                pm.name().to_string(),
            ));
        }
    }

    // 4. Fallback: only for well-formed names ≥ 3 chars
    if lower_pkg.len() >= 3 {
        return Some((
            pm.remove_cmd(lower_pkg),
            format!("Uninstall '{}' via {}", lower_pkg, pm.name()),
            pm.name().to_string(),
        ));
    }

    None
}

fn format_package_install(entry: &UniversalPackage, ctx: &SystemContext) -> (String, String, String) {
    match ctx.distro_family {
        DistroFamily::Arch => {
            if entry.is_aur && (ctx.pkg_manager == crate::context::PackageManager::Paru || ctx.pkg_manager == crate::context::PackageManager::Yay) {
                (format!("{} -S {}", ctx.pkg_manager.name(), entry.arch_pkg), format!("Install {} (AUR)", entry.desc), "AUR".to_string())
            } else if entry.is_aur {
                (format!("paru -S {} || sudo pacman -S {}", entry.arch_pkg, entry.arch_pkg), format!("Install {} (AUR)", entry.desc), "AUR".to_string())
            } else {
                (format!("sudo pacman -S {}", entry.arch_pkg), format!("Install {} (Pacman)", entry.desc), "Pacman".to_string())
            }
        }
        DistroFamily::Debian => {
            if entry.debian_pkg.starts_with("flatpak") || entry.debian_pkg.starts_with("curl") || entry.debian_pkg.starts_with("cargo") {
                (entry.debian_pkg.to_string(), format!("Install {}", entry.desc), "Install".to_string())
            } else {
                (format!("sudo apt install {}", entry.debian_pkg), format!("Install {} (APT)", entry.desc), "Apt".to_string())
            }
        }
        DistroFamily::Fedora => {
            if entry.fedora_pkg.starts_with("flatpak") || entry.fedora_pkg.starts_with("groupinstall") {
                (format!("sudo dnf {}", entry.fedora_pkg), format!("Install {} (DNF)", entry.desc), "Dnf".to_string())
            } else {
                (format!("sudo dnf install {}", entry.fedora_pkg), format!("Install {} (DNF)", entry.desc), "Dnf".to_string())
            }
        }
        _ => {
            (ctx.pkg_manager.install_cmd(entry.arch_pkg), format!("Install {} ({})", entry.desc, ctx.pkg_manager.name()), ctx.pkg_manager.name().to_string())
        }
    }
}

fn match_archive_extraction(lower: &str, ctx: &SystemContext) -> Option<CandidateSuggestion> {
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
