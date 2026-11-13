use super::{HeaderMap, Request};
use std::collections::HashMap;

/// Per-request parsing limits.
#[derive(Debug, Clone, Copy)]
pub struct ParseLimits {
    pub max_request_line: usize,
    pub max_header_size: usize,
    pub max_header_count: usize,
    pub max_body_size: usize,
}
impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_request_line: 8192,
            max_header_size: 16384,
            max_header_count: 100,
            max_body_size: 1_048_576,
        }
    }
}
/// Parsing failure suitable for mapping to an HTTP response.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("malformed request")]
    Malformed,
    #[error("unsupported method")]
    UnsupportedMethod,
    #[error("unsupported HTTP version")]
    Version,
    #[error("request too large")]
    TooLarge,
    #[error("chunked transfer encoding is unsupported")]
    Chunked,
    #[error("incomplete request")]
    Incomplete,
}
/// Parses one complete request from bytes. Trailing bytes are ignored for keep-alive callers.
pub fn parse_request(input: &[u8], limits: ParseLimits) -> Result<Request, ParseError> {
    let end = input
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or(ParseError::Incomplete)?;
    if end > limits.max_header_size {
        return Err(ParseError::TooLarge);
    }
    let head = std::str::from_utf8(&input[..end]).map_err(|_| ParseError::Malformed)?;
    let mut lines = head.split("\r\n");
    let line = lines.next().ok_or(ParseError::Malformed)?;
    if line.len() > limits.max_request_line {
        return Err(ParseError::TooLarge);
    }
    let mut parts = line.split_whitespace();
    let method = parts
        .next()
        .ok_or(ParseError::Malformed)?
        .parse()
        .map_err(|_| ParseError::UnsupportedMethod)?;
    let target = parts.next().ok_or(ParseError::Malformed)?;
    let version = parts.next().ok_or(ParseError::Malformed)?;
    if parts.next().is_some() {
        return Err(ParseError::Malformed);
    }
    if version != "HTTP/1.0" && version != "HTTP/1.1" {
        return Err(ParseError::Version);
    }
    let mut headers = HeaderMap::default();
    let mut count = 0;
    for l in lines {
        count += 1;
        if count > limits.max_header_count {
            return Err(ParseError::TooLarge);
        }
        let (n, v) = l.split_once(':').ok_or(ParseError::Malformed)?;
        if n.is_empty()
            || !n
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
        {
            return Err(ParseError::Malformed);
        }
        headers.insert(n.trim().to_string(), v.trim().to_string());
    }
    if headers
        .get("transfer-encoding")
        .is_some_and(|v| v.eq_ignore_ascii_case("chunked"))
    {
        return Err(ParseError::Chunked);
    }
    let length = match headers.get("content-length") {
        Some(v) => v.parse::<usize>().map_err(|_| ParseError::Malformed)?,
        None => 0,
    };
    if length > limits.max_body_size {
        return Err(ParseError::TooLarge);
    }
    let start = end + 4;
    if input.len() < start + length {
        return Err(ParseError::Incomplete);
    }
    let (path, q) = target.split_once('?').unwrap_or((target, ""));
    let mut query = HashMap::new();
    for pair in q.split('&').filter(|x| !x.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        query.insert(decode(k)?, decode(v)?);
    }
    Ok(Request {
        method,
        target: target.to_string(),
        path: path.to_string(),
        query,
        version: version.to_string(),
        headers,
        body: input[start..start + length].to_vec(),
    })
}
fn decode(s: &str) -> Result<String, ParseError> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if i + 2 >= b.len() {
                return Err(ParseError::Malformed);
            }
            let h = std::str::from_utf8(&b[i + 1..i + 3]).map_err(|_| ParseError::Malformed)?;
            out.push(u8::from_str_radix(h, 16).map_err(|_| ParseError::Malformed)?);
            i += 3
        } else {
            out.push(if b[i] == b'+' { b' ' } else { b[i] });
            i += 1
        }
    }
    String::from_utf8(out).map_err(|_| ParseError::Malformed)
}
