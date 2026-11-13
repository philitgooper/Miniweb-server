use crate::http::{Response, Status};
/// Returns the service health document.
pub fn health(_: crate::http::Request) -> Response {
    json(r#"{"server":"MiniWeb","status":"online","language":"Rust"}"#)
}
/// Returns service metadata.
pub fn info(_: crate::http::Request) -> Response {
    json(r#"{"name":"MiniWeb","protocol":"HTTP/1.1 subset"}"#)
}
/// Accepts a JSON API payload.
pub fn data(req: crate::http::Request) -> Response {
    if !req
        .headers
        .get("content-type")
        .is_some_and(|v| v.to_ascii_lowercase().starts_with("application/json"))
    {
        return Response::text(Status::BAD_REQUEST, "JSON required");
    }
    if serde_json::from_slice::<serde_json::Value>(&req.body).is_err() {
        return Response::text(Status::BAD_REQUEST, "Invalid JSON");
    }
    let mut r = Response::new(Status::CREATED);
    r.headers
        .insert("Content-Type".into(), "application/json".into());
    r.body = b"{\"accepted\":true}".to_vec();
    r
}
fn json(s: &str) -> Response {
    let mut r = Response::new(Status::OK);
    r.headers
        .insert("Content-Type".into(), "application/json".into());
    r.body = s.as_bytes().to_vec();
    r
}
