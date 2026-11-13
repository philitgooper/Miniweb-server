use crate::http::{Response, Status};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
/// Static file service constrained to a canonical root.
pub struct StaticFiles {
    root: PathBuf,
    max_file_size: u64,
}
impl StaticFiles {
    /// Opens a static root.
    pub fn new(root: &Path, max_file_size: u64) -> std::io::Result<Self> {
        Ok(Self {
            root: fs::canonicalize(root)?,
            max_file_size,
        })
    }
    /// Reads a file after traversal, dotfile, symlink, and size checks.
    pub fn get(&self, url_path: &str) -> Response {
        let decoded = match percent_decode(url_path) {
            Some(v) => v,
            None => return Response::text(Status::NOT_FOUND, "Not Found"),
        };
        let rel = decoded.trim_start_matches('/');
        let rel = if rel.is_empty() { "index.html" } else { rel };
        let p = Path::new(rel);
        if p.components().any(|c| !matches!(c, Component::Normal(_)))
            || p.components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        {
            return Response::text(Status::NOT_FOUND, "Not Found");
        }
        let joined = self.root.join(p);
        let canonical = match fs::canonicalize(joined) {
            Ok(v) => v,
            Err(_) => return Response::text(Status::NOT_FOUND, "Not Found"),
        };
        if !canonical.starts_with(&self.root) {
            return Response::text(Status::NOT_FOUND, "Not Found");
        }
        let meta = match fs::metadata(&canonical) {
            Ok(v) => v,
            Err(_) => return Response::text(Status::NOT_FOUND, "Not Found"),
        };
        if !meta.is_file() || meta.len() > self.max_file_size {
            return Response::text(Status::NOT_FOUND, "Not Found");
        }
        let body = match fs::read(&canonical) {
            Ok(v) => v,
            Err(_) => return Response::text(Status::NOT_FOUND, "Not Found"),
        };
        let mut r = Response::new(Status::OK);
        r.headers
            .insert("Content-Type".into(), mime(&canonical).into());
        r.body = body;
        r
    }
}
fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut o = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if i + 2 >= b.len() {
                return None;
            }
            o.push(u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).ok()?, 16).ok()?);
            i += 3
        } else {
            o.push(b[i]);
            i += 1
        }
    }
    String::from_utf8(o).ok()
}
fn mime(p: &Path) -> &'static str {
    match p
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}
