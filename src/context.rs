use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub cwd: PathBuf,
    pub files: Vec<String>,
    pub is_git_repo: bool,
    pub git_branch: Option<String>,
    pub git_modified_files: Vec<String>,
    pub os_id: String,
    pub shell: String,
    pub recent_history: Vec<String>,
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
        let os_id = Self::detect_os();
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
            os_id,
            shell,
            recent_history: sanitized_history,
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
        // Quick check if .git directory exists in hierarchy
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

        // Check git branch
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

        // Check modified files (porcelain)
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

    fn detect_os() -> String {
        if let Ok(contents) = fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if let Some(val) = line.strip_prefix("ID=") {
                    return val.trim_matches('"').to_string();
                }
            }
        }
        "cachyos".to_string()
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
}
