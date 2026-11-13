/// HTTP status and its standard reason phrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    pub code: u16,
    pub reason: &'static str,
}
impl Status {
    pub const OK: Self = Self {
        code: 200,
        reason: "OK",
    };
    pub const CREATED: Self = Self {
        code: 201,
        reason: "Created",
    };
    pub const NO_CONTENT: Self = Self {
        code: 204,
        reason: "No Content",
    };
    pub const BAD_REQUEST: Self = Self {
        code: 400,
        reason: "Bad Request",
    };
    pub const NOT_FOUND: Self = Self {
        code: 404,
        reason: "Not Found",
    };
    pub const METHOD_NOT_ALLOWED: Self = Self {
        code: 405,
        reason: "Method Not Allowed",
    };
    pub const REQUEST_TIMEOUT: Self = Self {
        code: 408,
        reason: "Request Timeout",
    };
    pub const PAYLOAD_TOO_LARGE: Self = Self {
        code: 413,
        reason: "Payload Too Large",
    };
    pub const HEADER_TOO_LARGE: Self = Self {
        code: 431,
        reason: "Request Header Fields Too Large",
    };
    pub const INTERNAL_ERROR: Self = Self {
        code: 500,
        reason: "Internal Server Error",
    };
    pub const NOT_IMPLEMENTED: Self = Self {
        code: 501,
        reason: "Not Implemented",
    };
    pub const HTTP_VERSION_NOT_SUPPORTED: Self = Self {
        code: 505,
        reason: "HTTP Version Not Supported",
    };
}
