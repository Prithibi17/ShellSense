use shellsense::config::Config;
use shellsense::protocol::{Request, Response};
use shellsense::AssistantEngine;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_or_default();
    let socket_path = Config::socket_path();

    println!("Starting shellsensd v0.1.0...");
    println!("Socket: {}", socket_path.display());
    println!("AI Provider: {} ({})", config.ai.provider, config.ai.endpoint);

    // Clean up stale socket if present
    if socket_path.exists() {
        if let Ok(_) = UnixStream::connect(&socket_path).await {
            eprintln!("Error: Another instance of shellsensd is already running on {}", socket_path.display());
            std::process::exit(1);
        } else {
            let _ = fs::remove_file(&socket_path);
        }
    }

    if let Some(parent) = socket_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let listener = UnixListener::bind(&socket_path)?;
    // Set socket permissions to user only (0600)
    let _ = fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600));

    let engine = Arc::new(AssistantEngine::new(config));

    // Handle shutdown gracefully
    let cleanup_path = socket_path.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\nShutting down shellsensd...");
        let _ = fs::remove_file(cleanup_path);
        std::process::exit(0);
    });

    println!("shellsensd is ready. Listening for client requests.");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let engine_ref = Arc::clone(&engine);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, engine_ref).await {
                        eprintln!("Client handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Listener accept error: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut stream: UnixStream,
    engine: Arc<AssistantEngine>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    if buf_reader.read_line(&mut line).await? > 0 {
        let request: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = Response::Error(format!("Invalid request format: {}", e));
                let json = serde_json::to_string(&err_resp)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                return Ok(());
            }
        };

        let response = engine.handle_request(request).await;
        let resp_json = serde_json::to_string(&response)?;
        writer.write_all(resp_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}
