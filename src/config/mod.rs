use serde::Deserialize;
use std::{fs, path::PathBuf};
/// Validated server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout: u64,
    pub static_dir: PathBuf,
    pub max_header_size: usize,
    pub max_body_size: usize,
    pub max_file_size: u64,
    pub worker_threads: usize,
    pub log_level: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            max_connections: 100,
            request_timeout: 30,
            static_dir: "public".into(),
            max_header_size: 16384,
            max_body_size: 1048576,
            max_file_size: 10485760,
            worker_threads: 4,
            log_level: "info".into(),
        }
    }
}
impl Config {
    /// Loads TOML then applies supported environment overrides.
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut c = Self::default();
        if let Ok(s) = fs::read_to_string("configs/default.toml") {
            c = toml::from_str(&s)?
        }
        macro_rules! envv {
            ($key:literal,$field:ident,$ty:ty) => {
                if let Ok(v) = std::env::var($key) {
                    c.$field = v.parse::<$ty>()?
                }
            };
        }
        if let Ok(v) = std::env::var("HOST") {
            c.host = v
        }
        envv!("PORT", port, u16);
        envv!("MAX_CONNECTIONS", max_connections, usize);
        envv!("REQUEST_TIMEOUT", request_timeout, u64);
        if let Ok(v) = std::env::var("STATIC_DIR") {
            c.static_dir = v.into()
        }
        envv!("MAX_HEADER_SIZE", max_header_size, usize);
        envv!("MAX_BODY_SIZE", max_body_size, usize);
        envv!("MAX_FILE_SIZE", max_file_size, u64);
        envv!("WORKER_THREADS", worker_threads, usize);
        if let Ok(v) = std::env::var("LOG_LEVEL") {
            c.log_level = v
        }
        if c.max_connections == 0
            || c.worker_threads == 0
            || c.request_timeout == 0
            || c.max_header_size < 128
        {
            return Err("invalid zero or undersized configuration limit".into());
        }
        Ok(c)
    }
}
