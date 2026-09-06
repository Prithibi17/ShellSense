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
