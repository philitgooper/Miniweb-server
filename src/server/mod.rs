mod async_server;
mod connection;
mod tcp;
use crate::config::Config;
/// Starts selected server mode.
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .windows(2)
        .find(|pair| pair[0] == "--mode")
        .map(|pair| pair[1].as_str())
        .or_else(|| args.iter().find_map(|arg| arg.strip_prefix("--mode=")))
        .unwrap_or("async");
    if mode == "threaded" {
        tokio::task::spawn_blocking(move || tcp::run(config))
            .await
            .map_err(|e| e.to_string())??;
    } else {
        async_server::run(config).await?
    }
    Ok(())
}

/// Serves a Tokio stream through MiniWeb's async connection implementation.
pub use async_server::serve_stream;
