use crate::{
    config::Config,
    http::{Method, ParseError, ParseLimits, Request, Response, Status},
    static_files::StaticFiles,
};
use std::path::Path;
/// Routes a parsed request to an API or static file handler.
pub fn respond(request: Request, config: &Config) -> Response {
    if request.method == Method::Options {
        let mut response = Response::new(Status::NO_CONTENT);
        response.headers.insert(
            "Allow".into(),
            "GET, POST, PUT, DELETE, HEAD, OPTIONS".into(),
        );
        return response;
    }
    let method = if request.method == Method::Head {
        Method::Get
    } else {
        request.method
    };
    match (method, request.path.as_str()) {
        (Method::Get, "/health") => crate::handlers::health(request),
        (Method::Get, "/api/status") => crate::handlers::health(request),
        (Method::Get, "/api/info") => crate::handlers::info(request),
        (Method::Post, "/api/data") => crate::handlers::data(request),
        (Method::Get, "/about") => static_response(&request, config, "/about.html"),
        (Method::Get, "/") => static_response(&request, config, "/"),
        (Method::Get, path) if !path.starts_with("/api/") => {
            static_response(&request, config, path)
        }
        (_, "/health" | "/api/status" | "/api/info" | "/api/data" | "/about" | "/") => {
            let mut r = Response::text(Status::METHOD_NOT_ALLOWED, "Method Not Allowed");
            r.headers.insert(
                "Allow".into(),
                if request.path == "/api/data" {
                    "POST".into()
                } else {
                    "GET, HEAD, OPTIONS".into()
                },
            );
            r
        }
        _ => Response::text(Status::NOT_FOUND, "Not Found"),
    }
}
fn static_response(req: &Request, cfg: &Config, path: &str) -> Response {
    let root = Path::new(&cfg.static_dir);
    match StaticFiles::new(root, cfg.max_file_size) {
        Ok(files) => files.get(if req.method == Method::Head && path == "/" {
            "/"
        } else {
            path
        }),
        Err(_) => Response::text(Status::INTERNAL_ERROR, "Internal Server Error"),
    }
}
/// Converts parser errors to safe protocol responses.
pub fn parse_error(error: ParseError) -> Response {
    let status = match error {
        ParseError::TooLarge => Status::HEADER_TOO_LARGE,
        ParseError::BodyTooLarge => Status::PAYLOAD_TOO_LARGE,
        ParseError::Chunked => Status::NOT_IMPLEMENTED,
        ParseError::Version => Status::HTTP_VERSION_NOT_SUPPORTED,
        _ => Status::BAD_REQUEST,
    };
    Response::text(status, status.reason)
}
/// Uses configured parser limits.
pub fn limits(config: &Config) -> ParseLimits {
    ParseLimits {
        max_header_size: config.max_header_size,
        max_body_size: config.max_body_size,
        ..ParseLimits::default()
    }
}
