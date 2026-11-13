use crate::{
    config::Config,
    http::{Method, parse_request},
    server::connection,
};
use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        mpsc::{Receiver, sync_channel},
    },
    thread,
    time::Duration,
};
type Job = TcpStream;
/// Runs Stage A with a bounded queue and fixed worker threads.
pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr)?;
    tracing::info!(mode="threaded",bind=%addr,workers=config.worker_threads,"server started");
    if config.host == "0.0.0.0" {
        tracing::warn!("server is exposed on all network interfaces");
    }
    let (tx, rx) = sync_channel::<Job>(config.max_connections);
    let rx = Arc::new(std::sync::Mutex::new(rx));
    for _ in 0..config.worker_threads {
        let rx = rx.clone();
        let cfg = config.clone();
        thread::spawn(move || worker(rx, cfg));
    }
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let _ = s.set_nodelay(true);
                if tx.send(s).is_err() {
                    break;
                }
            }
            Err(e) => tracing::warn!(error=%e,"accept failed"),
        }
    }
    Ok(())
}
fn worker(rx: Arc<std::sync::Mutex<Receiver<Job>>>, cfg: Config) {
    loop {
        let stream = {
            let lock = match rx.lock() {
                Ok(l) => l,
                Err(_) => return,
            };
            lock.recv()
        };
        match stream {
            Ok(s) => {
                let _ = catch_unwind(AssertUnwindSafe(|| handle(s, &cfg)));
            }
            Err(_) => return,
        }
    }
}
fn handle(stream: TcpStream, cfg: &Config) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(cfg.request_timeout)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(cfg.request_timeout)));
    let mut reader = BufReader::new(stream);
    loop {
        let mut buf = Vec::with_capacity(cfg.max_header_size.min(65536));
        while !buf.ends_with(b"\r\n\r\n") {
            let mut b = [0];
            match reader.read(&mut b) {
                Ok(0) | Err(_) => return,
                Ok(_) => buf.push(b[0]),
            }
            if buf.len() > cfg.max_header_size + 4 {
                let _ = reader.get_mut().write_all(
                    &connection::parse_error(crate::http::ParseError::TooLarge).serialize(false),
                );
                return;
            }
        }
        let head = String::from_utf8_lossy(&buf);
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
            let response = connection::parse_error(crate::http::ParseError::BodyTooLarge);
            let _ = reader.get_mut().write_all(&response.serialize(false));
            return;
        }
        let mut body = vec![0; length];
        if reader.read_exact(&mut body).is_err() {
            return;
        }
        buf.extend_from_slice(&body);
        let req = match parse_request(&buf, connection::limits(cfg)) {
            Ok(r) => r,
            Err(e) => {
                let _ = reader
                    .get_mut()
                    .write_all(&connection::parse_error(e).serialize(false));
                return;
            }
        };
        let peer = reader
            .get_ref()
            .peer_addr()
            .map_or_else(|_| "unknown".to_owned(), |a| a.to_string());
        let method = req.method;
        let path = req.path.clone();
        let started = std::time::Instant::now();
        let close = req
            .headers
            .get("connection")
            .is_some_and(|v| v.eq_ignore_ascii_case("close"))
            || req.version == "HTTP/1.0";
        let head_only = req.method == Method::Head;
        let mut response = if req.method == Method::Options {
            let mut r = crate::http::Response::new(crate::http::Status::NO_CONTENT);
            r.headers.insert(
                "Allow".into(),
                "GET, POST, PUT, DELETE, HEAD, OPTIONS".into(),
            );
            r
        } else {
            connection::respond(req, cfg)
        };
        if close {
            response.headers.insert("Connection".into(), "close".into());
        }
        let status = response.status.code;
        if reader
            .get_mut()
            .write_all(&response.serialize(head_only))
            .is_err()
        {
            return;
        }
        tracing::info!(peer=%peer,method=%method,path=%path,status,duration_ms=started.elapsed().as_millis() as u64,"request complete");
        if close {
            return;
        }
    }
}
