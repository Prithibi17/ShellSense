use clap::{Parser, Subcommand};
use shellsense::config::Config;
use shellsense::protocol::{
    ExplainRequest, ModelsRequest, RecordRequest, Request, Response, StatusRequest,
    SuggestRequest, SuggestTrigger,
};
use shellsense::AssistantEngine;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Parser)]
#[command(name = "shellsense")]
#[command(about = "ShellSense: IntelliSense-like terminal command assistant and autocomplete")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Suggest command(s) based on natural language or partial input
    Suggest {
        /// The query or intent typed by the user
        input: String,

        /// Current working directory
        #[arg(long)]
        cwd: Option<String>,

        /// Shell type (default: fish)
        #[arg(long)]
        shell: Option<String>,

        /// Print only the top command string (for shell commandline replacement)
        #[arg(long, short)]
        raw: bool,

        /// Print all candidate command strings (one per line) for cycling
        #[arg(long)]
        raw_all: bool,

        /// Output full response as JSON
        #[arg(long)]
        json: bool,

        /// Format for ghost-text display
        #[arg(long)]
        ghost: bool,

        /// Recent commands for context
        #[arg(long)]
        history: Vec<String>,
    },

    /// Explain what a Linux command does and check its safety risk
    Explain {
        /// Command to explain
        command: String,

        /// Output full response as JSON
        #[arg(long)]
        json: bool,
    },

    /// List installed AI models
    Models {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check daemon status, connection, and health
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Record an accepted command for local learning
    Record {
        /// The input intent
        #[arg(long)]
        input: String,

        /// The accepted command
        #[arg(long)]
        command: String,
    },

    /// Clear daemon suggestion cache
    ClearCache,

    /// Update ShellSense to the latest release from GitHub
    Update {
        /// Force re-installation even if already on the latest version
        #[arg(long, short)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let socket_path = Config::socket_path();

    match cli.command {
        Commands::Suggest {
            input,
            cwd,
            shell,
            raw,
            raw_all,
            json,
            ghost,
            history,
        } => {
            let req = Request::Suggest(SuggestRequest {
                input: input.clone(),
                cwd: cwd.clone(),
                shell: shell.clone(),
                history: if history.is_empty() { None } else { Some(history) },
                trigger: if ghost {
                    SuggestTrigger::Ghost
                } else {
                    SuggestTrigger::Manual
                },
            });

            let resp = send_request_or_fallback(&socket_path, req, || {
                let config = Config::load_or_default();
                let engine = AssistantEngine::new(config);
                let fallback_req = SuggestRequest {
                    input,
                    cwd,
                    shell,
                    history: None,
                    trigger: SuggestTrigger::Manual,
                };
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        Response::Suggest(engine.suggest(fallback_req).await)
                    })
                })
            })
            .await?;

            if let Response::Suggest(sug_resp) = resp {
                if json {
                    println!("{}", serde_json::to_string_pretty(&sug_resp)?);
                } else if raw {
                    if let Some(first) = sug_resp.suggestions.first() {
                        print!("{}", first.command);
                    }
                } else if raw_all {
                    for item in &sug_resp.suggestions {
                        let cat = item.category.as_deref().unwrap_or("Linux");
                        let is_dest = if item.risk == shellsense::protocol::RiskLevel::Destructive { "dest" } else { "ok" };
                        println!("{}\t{}\t{}\t{}", cat, item.command, item.description, is_dest);
                    }
                } else if ghost {
                    if let Some(first) = sug_resp.suggestions.first() {
                        let cat = first.category.as_deref().unwrap_or("Linux");
                        let is_dest = if first.risk == shellsense::protocol::RiskLevel::Destructive { "dest" } else { "ok" };
                        println!("{}\t{}\t{}\t{}", cat, first.command, first.description, is_dest);
                    }
                } else {
                    // Pretty human-readable output
                    if sug_resp.suggestions.is_empty() {
                        eprintln!("No suggestions found.");
                    } else {
                        for (idx, item) in sug_resp.suggestions.iter().enumerate() {
                            let risk_tag = match item.risk {
                                shellsense::protocol::RiskLevel::Low => "",
                                shellsense::protocol::RiskLevel::Medium => " [Notice: modifies state]",
                                shellsense::protocol::RiskLevel::Destructive => " [⚠ Potentially destructive]",
                            };
                            println!("{}. {}{}", idx + 1, item.command, risk_tag);
                            println!("   {} (conf: {:.0}%, src: {})", item.description, item.confidence * 100.0, item.source);
                            if let Some(ref w) = item.warning {
                                println!("   {}", w);
                            }
                        }
                    }
                }
            } else if let Response::Error(err) = resp {
                eprintln!("Error: {}", err);
            }
        }

        Commands::Explain { command, json } => {
            let req = Request::Explain(ExplainRequest {
                command: command.clone(),
            });

            let resp = send_request_or_fallback(&socket_path, req, || {
                let config = Config::load_or_default();
                let engine = AssistantEngine::new(config);
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        Response::Explain(engine.explain(&command).await)
                    })
                })
            })
            .await?;

            if let Response::Explain(exp_resp) = resp {
                if json {
                    println!("{}", serde_json::to_string_pretty(&exp_resp)?);
                } else {
                    println!("Command: {}", exp_resp.command);
                    println!("Explanation: {}", exp_resp.explanation);
                    if let Some(ref w) = exp_resp.warning {
                        println!("Warning: {}", w);
                    }
                }
            }
        }

        Commands::Models { json } => {
            let req = Request::Models(ModelsRequest {});
            let resp = send_request(&socket_path, req).await?;
            if let Response::Models(m_resp) = resp {
                if json {
                    println!("{}", serde_json::to_string_pretty(&m_resp)?);
                } else {
                    println!("Provider: {}", m_resp.provider);
                    println!("Active Model: {}", m_resp.active_model.unwrap_or_else(|| "none".to_string()));
                    println!("Installed Models:");
                    if m_resp.models.is_empty() {
                        println!("  (No models installed. Pull a model using: ollama pull qwen2.5-coder:1.5b)");
                    } else {
                        for m in m_resp.models {
                            println!("  - {}", m);
                        }
                    }
                }
            }
        }

        Commands::Status { json } => {
            let req = Request::Status(StatusRequest {});
            match send_request(&socket_path, req).await {
                Ok(Response::Status(s_resp)) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&s_resp)?);
                    } else {
                        println!("ShellSense Daemon: ● Running");
                        println!("Socket: {}", s_resp.socket_path);
                        println!("Uptime: {}s", s_resp.uptime_secs);
                        println!("AI Available: {}", if s_resp.ai_available { "Yes" } else { "No (Ollama stopped or no models)" });
                        println!("Active Model: {}", s_resp.active_model.unwrap_or_else(|| "none".to_string()));
                        println!("Cache Items: {}", s_resp.cache_size);
                    }
                }
                _ => {
                    if json {
                        println!("{{\"running\": false}}");
                    } else {
                        println!("ShellSense Daemon: ○ Stopped");
                        println!("Socket: {}", socket_path.display());
                        println!("Note: Commands will still use local deterministic engine fallback.");
                    }
                }
            }
        }

        Commands::Record { input, command } => {
            let req = Request::Record(RecordRequest { input, command });
            let _ = send_request(&socket_path, req).await;
        }

        Commands::ClearCache => {
            let req = Request::ClearCache;
            match send_request(&socket_path, req).await {
                Ok(_) => println!("Suggestion cache cleared."),
                Err(e) => eprintln!("Could not reach daemon: {}", e),
            }
        }

        Commands::Update { force } => {
            run_self_update(force).await?;
        }
    }

    Ok(())
}

async fn send_request(socket_path: &PathBuf, req: Request) -> Result<Response, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let req_json = serde_json::to_string(&req)?;

    stream.write_all(req_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let (reader, _) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;

    let resp: Response = serde_json::from_str(&line)?;
    Ok(resp)
}

async fn send_request_or_fallback<F>(
    socket_path: &PathBuf,
    req: Request,
    fallback: F,
) -> Result<Response, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Response,
{
    match send_request(socket_path, req).await {
        Ok(resp) => Ok(resp),
        Err(_) => {
            // Daemon is not running: seamlessly use in-process engine
            Ok(fallback())
        }
    }
}

async fn run_self_update(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("🔍 Checking for ShellSense updates (current version: v{})...", current_version);

    let client = reqwest::Client::builder()
        .user_agent("ShellSense-Updater")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let api_url = "https://api.github.com/repos/Prithibi17/ShellSense/releases/latest";
    let resp = client.get(api_url).send().await;

    let (latest_tag, download_url) = match resp {
        Ok(res) if res.status().is_success() => {
            let json: serde_json::Value = res.json().await?;
            let tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
            let mut dl = format!(
                "https://github.com/Prithibi17/ShellSense/releases/download/v{}/shellsense-linux-x86_64.tar.gz",
                tag
            );
            if let Some(assets) = json["assets"].as_array() {
                for a in assets {
                    if let Some(name) = a["name"].as_str() {
                        if name.contains("linux-x86_64") && name.ends_with(".tar.gz") {
                            if let Some(browser_dl) = a["browser_download_url"].as_str() {
                                dl = browser_dl.to_string();
                                break;
                            }
                        }
                    }
                }
            }
            (tag, dl)
        }
        _ => {
            eprintln!("Notice: GitHub API unreachable or rate-limited. Falling back to latest GitHub release...");
            (
                "latest".to_string(),
                "https://github.com/Prithibi17/ShellSense/releases/latest/download/shellsense-linux-x86_64.tar.gz".to_string(),
            )
        }
    };

    if !force && !latest_tag.is_empty() && latest_tag != "latest" && latest_tag == current_version {
        println!("✓ ShellSense is already on the latest version (v{}).", current_version);
        println!("Tip: Run 'ss update --force' to force re-download and re-install.");
        return Ok(());
    }

    println!("⚡ Updating ShellSense to v{}...", if latest_tag == "latest" { "latest" } else { &latest_tag });
    println!("   Downloading {}", download_url);

    let dl_resp = client.get(&download_url).send().await?;
    if !dl_resp.status().is_success() {
        println!("Binary download returned {}, falling back to universal installer...", dl_resp.status());
        let status = std::process::Command::new("bash")
            .arg("-c")
            .arg("curl -fsSL https://raw.githubusercontent.com/Prithibi17/ShellSense/main/install.sh | bash")
            .status()?;
        if !status.success() {
            return Err("Universal installer failed to complete update".into());
        }
        return Ok(());
    }

    let bytes = dl_resp.bytes().await?;
    let tmp_dir = std::env::temp_dir().join(format!("shellsense-update-{}", std::process::id()));
    tokio::fs::create_dir_all(&tmp_dir).await?;
    let tar_path = tmp_dir.join("shellsense.tar.gz");
    tokio::fs::write(&tar_path, &bytes).await?;

    // Extract archive using tar
    let extract_status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&tar_path)
        .arg("-C")
        .arg(&tmp_dir)
        .status()?;

    if !extract_status.success() {
        let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
        return Err("Failed to extract update tarball archive".into());
    }

    // Stop daemon during binary replacement
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "stop", "shellsense.service"])
        .status();
    let _ = std::process::Command::new("pkill")
        .arg("-f")
        .arg("shellsensd")
        .status();

    let home = dirs::home_dir().ok_or("Could not resolve user home directory")?;
    let bin_dir = home.join(".local/bin");
    tokio::fs::create_dir_all(&bin_dir).await?;

    let new_cli = tmp_dir.join("shellsense");
    let new_daemon = tmp_dir.join("shellsensd");

    if new_cli.exists() {
        tokio::fs::copy(&new_cli, bin_dir.join("shellsense")).await?;
        let _ = std::process::Command::new("chmod")
            .args(["+x", bin_dir.join("shellsense").to_str().unwrap()])
            .status();
        let _ = std::process::Command::new("ln")
            .args(["-sf", bin_dir.join("shellsense").to_str().unwrap(), bin_dir.join("ss").to_str().unwrap()])
            .status();
    }
    if new_daemon.exists() {
        tokio::fs::copy(&new_daemon, bin_dir.join("shellsensd")).await?;
        let _ = std::process::Command::new("chmod")
            .args(["+x", bin_dir.join("shellsensd").to_str().unwrap()])
            .status();
    }

    // Also update in ~/.cargo/bin if present
    let cargo_bin = home.join(".cargo/bin");
    if cargo_bin.exists() {
        if new_cli.exists() {
            let _ = tokio::fs::copy(&new_cli, cargo_bin.join("shellsense")).await;
            let _ = std::process::Command::new("ln")
                .args(["-sf", cargo_bin.join("shellsense").to_str().unwrap(), cargo_bin.join("ss").to_str().unwrap()])
                .status();
        }
        if new_daemon.exists() {
            let _ = tokio::fs::copy(&new_daemon, cargo_bin.join("shellsensd")).await;
        }
    }

    // Refresh shell integration scripts
    let fish_dest = home.join(".config/fish/conf.d/shellsense.fish");
    if fish_dest.parent().map_or(false, |p| p.exists()) {
        if let Ok(resp) = client.get("https://raw.githubusercontent.com/Prithibi17/ShellSense/main/fish/shellsense.fish").send().await {
            if let Ok(text) = resp.text().await {
                let _ = tokio::fs::write(&fish_dest, text).await;
            }
        }
    }
    let bash_dest = home.join(".config/shellsense/shellsense.bash");
    if bash_dest.parent().map_or(false, |p| p.exists()) {
        if let Ok(resp) = client.get("https://raw.githubusercontent.com/Prithibi17/ShellSense/main/bash/shellsense.bash").send().await {
            if let Ok(text) = resp.text().await {
                let _ = tokio::fs::write(&bash_dest, text).await;
            }
        }
    }
    let zsh_dest = home.join(".config/shellsense/shellsense.zsh");
    if zsh_dest.parent().map_or(false, |p| p.exists()) {
        if let Ok(resp) = client.get("https://raw.githubusercontent.com/Prithibi17/ShellSense/main/zsh/shellsense.zsh").send().await {
            if let Ok(text) = resp.text().await {
                let _ = tokio::fs::write(&zsh_dest, text).await;
            }
        }
    }

    // Restart daemon
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "restart", "shellsense.service"])
        .status();

    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    println!("\n🚀 ShellSense successfully updated to v{}!", if latest_tag == "latest" { "latest" } else { &latest_tag });
    println!("   Binaries updated in: {}", bin_dir.display());
    println!("   Active background daemon restarted.");
    println!("   Shell integrations refreshed.");

    Ok(())
}
