use std::path::PathBuf;
use terminal_assistant::context::{sanitize_text, SystemContext};
use terminal_assistant::deterministic::match_deterministic;
use terminal_assistant::protocol::RiskLevel;
use terminal_assistant::safety::{check_safety, correct_distro_command};

fn make_context(files: Vec<&str>, is_git: bool) -> SystemContext {
    SystemContext {
        cwd: PathBuf::from("/test/project"),
        files: files.into_iter().map(String::from).collect(),
        is_git_repo: is_git,
        git_branch: if is_git { Some("main".into()) } else { None },
        git_modified_files: Vec::new(),
        os_id: "cachyos".into(),
        shell: "fish".into(),
        recent_history: Vec::new(),
        ..SystemContext::default()
    }
}

#[test]
fn test_destructive_command_detection() {
    let test_cases = vec![
        ("rm -rf /home/user/downloads", RiskLevel::Destructive),
        ("sudo mkfs.btrfs /dev/sdb1", RiskLevel::Destructive),
        ("sudo dd if=/dev/urandom of=/dev/nvme0n1 bs=4M", RiskLevel::Destructive),
        ("sudo pacman -Rns discord", RiskLevel::Destructive),
        ("sudo wipefs -a /dev/sda", RiskLevel::Destructive),
        ("sudo shutdown -h now", RiskLevel::Destructive),
    ];

    for (cmd, expected_risk) in test_cases {
        let rep = check_safety(cmd);
        assert_eq!(rep.risk, expected_risk, "Failed on command: {}", cmd);
        assert!(rep.warning.is_some(), "Expected warning for: {}", cmd);
    }
}

#[test]
fn test_distro_correction_to_arch() {
    let (c1, changed1) = correct_distro_command("apt install google-chrome-stable");
    assert!(changed1);
    assert_eq!(c1, "sudo pacman -S google-chrome-stable");

    let (c2, changed2) = correct_distro_command("sudo apt update");
    assert!(changed2);
    assert_eq!(c2, "sudo pacman -Sy");

    let (c3, changed3) = correct_distro_command("sudo dnf install neovim");
    assert!(changed3);
    assert_eq!(c3, "sudo pacman -S neovim");
}

#[test]
fn test_privacy_sanitization() {
    let raw = "export OPENAI_API_KEY=sk-test1234567890abcdef && export TOKEN=super_secret_val";
    let sanitized = sanitize_text(raw);
    assert!(!sanitized.contains("sk-test1234567890abcdef"));
    assert!(!sanitized.contains("super_secret_val"));
    assert!(sanitized.contains("[REDACTED]"));
}

#[test]
fn test_deterministic_cachyos_rules() {
    let ctx = make_context(vec![], false);

    let sugs = match_deterministic("instal chrome", &ctx);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "paru -S google-chrome");

    let sugs = match_deterministic("check nvidia", &ctx);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "nvidia-smi");

    let sugs = match_deterministic("restart audio", &ctx);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "systemctl --user restart pipewire pipewire-pulse wireplumber");

    let sugs = match_deterministic("what is using port 3000", &ctx);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "ss -ltnp | grep ':3000'");

    let sugs = match_deterministic("show disks", &ctx);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "lsblk -o NAME,SIZE,FSTYPE,MOUNTPOINTS");
}

#[test]
fn test_path_aware_heuristics() {
    let ctx_zip = make_context(vec!["archive.zip", "report.pdf"], false);
    let sugs = match_deterministic("extract archive", &ctx_zip);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "unzip \"archive.zip\"");

    let ctx_node = make_context(vec!["package.json", "node_modules"], false);
    let sugs = match_deterministic("run project", &ctx_node);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "npm run dev");

    let ctx_rust = make_context(vec!["Cargo.toml", "src"], false);
    let sugs = match_deterministic("run project", &ctx_rust);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "cargo run");
}

#[test]
fn test_git_heuristics() {
    let ctx_git = make_context(vec![], true);
    let sugs = match_deterministic("save changes", &ctx_git);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "git add .");

    let sugs = match_deterministic("undo last commit", &ctx_git);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "git reset --soft HEAD~1");
}

#[test]
fn test_universal_distro_adaptation() {
    use terminal_assistant::context::{DistroFamily, PackageManager};

    // 1. Ubuntu / Debian context
    let mut ctx_ubuntu = make_context(vec![], false);
    ctx_ubuntu.distro_family = DistroFamily::Debian;
    ctx_ubuntu.pkg_manager = PackageManager::Apt;

    let sugs = match_deterministic("instal chrome", &ctx_ubuntu);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo apt install google-chrome-stable");

    let sugs = match_deterministic("update system", &ctx_ubuntu);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo apt update && sudo apt upgrade");

    let sugs = match_deterministic("clean orphans", &ctx_ubuntu);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo apt autoremove");

    // 2. Fedora context
    let mut ctx_fedora = make_context(vec![], false);
    ctx_fedora.distro_family = DistroFamily::Fedora;
    ctx_fedora.pkg_manager = PackageManager::Dnf;

    let sugs = match_deterministic("instal chrome", &ctx_fedora);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo dnf install google-chrome-stable");

    let sugs = match_deterministic("update system", &ctx_fedora);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo dnf upgrade");

    let sugs = match_deterministic("clean orphans", &ctx_fedora);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo dnf autoremove");

    // 3. Alpine Linux context
    let mut ctx_alpine = make_context(vec![], false);
    ctx_alpine.distro_family = DistroFamily::Alpine;
    ctx_alpine.pkg_manager = PackageManager::Apk;

    let sugs = match_deterministic("update system", &ctx_alpine);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo apk update && sudo apk upgrade");
}

#[test]
fn test_universal_hardware_adaptation() {
    use terminal_assistant::context::{AudioSystem, FormFactor, GpuVendor};

    // 1. AMD Radeon GPU
    let mut ctx_amd = make_context(vec![], false);
    ctx_amd.gpu_vendor = GpuVendor::Amd;
    let sugs = match_deterministic("check gpu", &ctx_amd);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "radeontop");

    // 2. Intel Arc / iGPU
    let mut ctx_intel = make_context(vec![], false);
    ctx_intel.gpu_vendor = GpuVendor::Intel;
    let sugs = match_deterministic("check gpu", &ctx_intel);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "sudo intel_gpu_top");

    // 3. PulseAudio
    let mut ctx_pulse = make_context(vec![], false);
    ctx_pulse.audio_system = AudioSystem::Pulseaudio;
    let sugs = match_deterministic("restart audio", &ctx_pulse);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "pulseaudio -k && pulseaudio --start");

    // 4. Non-ASUS Generic Laptop (universal powerprofilesctl)
    let mut ctx_generic_laptop = make_context(vec![], false);
    ctx_generic_laptop.form_factor = FormFactor::Laptop;
    ctx_generic_laptop.has_asusctl = false;
    let sugs = match_deterministic("fan turbo", &ctx_generic_laptop);
    assert!(!sugs.is_empty());
    assert_eq!(sugs[0].command, "powerprofilesctl set performance");
}

