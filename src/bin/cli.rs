use clap::{Parser, Subcommand};
use std::path::PathBuf;
use terminal_assistant::config::Config;
use terminal_assistant::protocol::{
    ExplainRequest, ModelsRequest, RecordRequest, Request, Response, StatusRequest,
    SuggestRequest, SuggestTrigger,
};
use terminal_assistant::AssistantEngine;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Parser)]
#[command(name = "terminal-assistant")]
#[command(about = "AI-powered terminal command assistant and autocomplete client")]
#[command(version = "0.1.0")]
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
                        let is_dest = if item.risk == terminal_assistant::protocol::RiskLevel::Destructive { "dest" } else { "ok" };
                        println!("{}\t{}\t{}\t{}", cat, item.command, item.description, is_dest);
                    }
                } else if ghost {
                    if let Some(first) = sug_resp.suggestions.first() {
                        let cat = first.category.as_deref().unwrap_or("Linux");
                        let is_dest = if first.risk == terminal_assistant::protocol::RiskLevel::Destructive { "dest" } else { "ok" };
                        println!("{}\t{}\t{}\t{}", cat, first.command, first.description, is_dest);
                    }
                } else {
                    // Pretty human-readable output
                    if sug_resp.suggestions.is_empty() {
                        eprintln!("No suggestions found.");
                    } else {
                        for (idx, item) in sug_resp.suggestions.iter().enumerate() {
                            let risk_tag = match item.risk {
                                terminal_assistant::protocol::RiskLevel::Low => "",
                                terminal_assistant::protocol::RiskLevel::Medium => " [Notice: modifies state]",
                                terminal_assistant::protocol::RiskLevel::Destructive => " [⚠ Potentially destructive]",
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
                        println!("Terminal Assistant Daemon: ● Running");
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
                        println!("Terminal Assistant Daemon: ○ Stopped");
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
