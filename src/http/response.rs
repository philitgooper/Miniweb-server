use super::Status;
use std::collections::BTreeMap;
/// HTTP response ready for serialization.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: Status,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}
impl Response {
    /// Creates an empty response with a status.
    pub fn new(status: Status) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }
    /// Creates a plain text response.
    pub fn text(status: Status, body: impl Into<String>) -> Self {
        let mut r = Self::new(status);
        r.headers
            .insert("Content-Type".into(), "text/plain; charset=utf-8".into());
        r.body = body.into().into_bytes();
        r
    }
    /// Serializes the response using HTTP/1.1 framing.
    pub fn serialize(&self, head_only: bool) -> Vec<u8> {
        let mut out =
            format!("HTTP/1.1 {} {}\r\n", self.status.code, self.status.reason).into_bytes();
        let mut headers = self.headers.clone();
        headers.entry("Date".into()).or_insert_with(http_date);
        headers
            .entry("Server".into())
            .or_insert_with(|| "MiniWeb/0.1".into());
        headers
            .entry("Content-Length".into())
            .or_insert_with(|| self.body.len().to_string());
        headers
            .entry("Connection".into())
            .or_insert_with(|| "keep-alive".into());
        for (k, v) in headers {
            out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes())
        }
        out.extend_from_slice(b"\r\n");
        if !head_only {
            out.extend_from_slice(&self.body)
        }
        out
    }
}

fn http_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (seconds / 86_400) as i64;
    let day_seconds = seconds % 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let weekday =
        ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][(days + 4).rem_euclid(7) as usize];
    let month_name = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][(month - 1) as usize];
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    format!("{weekday}, {day:02} {month_name} {year:04} {hour:02}:{minute:02}:{second:02} GMT")
}
