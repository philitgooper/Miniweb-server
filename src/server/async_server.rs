use crate::{
    config::Config,
    http::{Method, parse_request},
    server::connection,
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Semaphore,
    task::JoinSet,
    time::{Duration, Instant, timeout_at},
};
/// Runs the Tokio HTTP server.
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(mode="async",bind=%addr,"server started");
    if config.host == "0.0.0.0" {
        tracing::warn!("server is exposed on all network interfaces")
    }
    let gate = Arc::new(Semaphore::new(config.max_connections));
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, peer) = accepted?;
                let permit = tokio::select! {
                    permit = gate.clone().acquire_owned() => permit?,
                    _ = shutdown_signal() => break,
                };
                let cfg = config.clone();
                tasks.spawn(async move {
                    let _permit = permit;
                    if let Err(error) = serve(stream, peer.to_string(), cfg).await {
                        tracing::warn!(peer=%peer, error=%error, "connection failed");
                    }
                });
            }
            _ = shutdown_signal() => break,
        }
    }
    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            tracing::warn!(error=%error, "connection task failed");
        }
    }
    Ok(())
}
async fn serve(stream: TcpStream, peer: String, cfg: Config) -> std::io::Result<()> {
    let _ = stream.set_nodelay(true);
    let (mut reader, mut writer) = stream.into_split();
    loop {
        let deadline = Instant::now() + Duration::from_secs(cfg.request_timeout);
        let mut buf = Vec::with_capacity(cfg.max_header_size.min(65536));
        let mut byte = [0u8; 1];
        let head_end = loop {
            let n = match timeout_at(deadline, reader.read(&mut byte)).await {
                Ok(Ok(n)) => n,
                _ => return Ok(()),
            };
            if n == 0 {
                return Ok(());
            }
            buf.push(byte[0]);
            if buf.len() > cfg.max_header_size + 4 {
                let out =
                    connection::parse_error(crate::http::ParseError::TooLarge).serialize(false);
                writer.write_all(&out).await?;
                return Ok(());
            }
            if buf.ends_with(b"\r\n\r\n") {
                break buf.len();
            }
        };
        let head = String::from_utf8_lossy(&buf[..head_end]);
        let length = head
            .lines()
            .find_map(|l| {
                let (n, v) = l.split_once(':')?;
                if n.eq_ignore_ascii_case("content-length") {
                    v.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        if length > cfg.max_body_size {
            writer
                .write_all(
                    &connection::parse_error(crate::http::ParseError::BodyTooLarge)
                        .serialize(false),
                )
                .await?;
            return Ok(());
        }
        let mut body = vec![0; length];
        if length > 0
            && timeout_at(deadline, reader.read_exact(&mut body))
                .await
                .is_err()
        {
            return Ok(());
        }
        buf.extend_from_slice(&body);
        let req = match parse_request(&buf, connection::limits(&cfg)) {
            Ok(r) => r,
            Err(e) => {
                writer
                    .write_all(&connection::parse_error(e).serialize(false))
                    .await?;
                return Ok(());
            }
        };
        let close = req
            .headers
            .get("connection")
            .is_some_and(|v| v.eq_ignore_ascii_case("close"))
            || req.version == "HTTP/1.0";
        let head_only = req.method == Method::Head;
        let method = req.method;
        let path = req.path.clone();
        let start = std::time::Instant::now();
        let mut response = if method == Method::Options {
            let mut r = crate::http::Response::new(crate::http::Status::NO_CONTENT);
            r.headers.insert(
                "Allow".into(),
                "GET, POST, PUT, DELETE, HEAD, OPTIONS".into(),
            );
            r
        } else {
            connection::respond(req, &cfg)
        };
        if close {
            response.headers.insert("Connection".into(), "close".into());
        }
        let out = response.serialize(head_only);
        writer.write_all(&out).await?;
        tracing::info!(peer=%peer,method=%method,path=%path,status=response.status.code,duration_ms=start.elapsed().as_millis() as u64,"request complete");
        if close {
            return Ok(());
        }
    }
}

/// Serves one accepted stream; exposed to support embedding and integration tests.
pub async fn serve_stream(stream: TcpStream, config: Config) -> std::io::Result<()> {
    let peer = stream
        .peer_addr()
        .map_or_else(|_| "unknown".to_owned(), |addr| addr.to_string());
    serve(stream, peer, config).await
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(error) => {
                tracing::error!(error=%error, "could not register SIGTERM handler");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
