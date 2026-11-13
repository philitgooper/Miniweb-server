use miniweb_server::http::{Method, ParseError, ParseLimits, Response, Status, parse_request};

#[test]
fn parses_methods_query_and_case_insensitive_headers() {
    let raw =
        b"POST /api/data?name=Rust+lang HTTP/1.1\r\nhOsT: localhost\r\nContent-Length: 2\r\n\r\n{}";
    let request = parse_request(raw, ParseLimits::default()).unwrap();
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.path, "/api/data");
    assert_eq!(request.query_param("name"), Some("Rust lang"));
    assert_eq!(request.headers.get("HOST"), Some("localhost"));
    assert_eq!(request.body, b"{}");
}

#[test]
fn maps_bad_and_oversized_requests_without_panicking() {
    assert_eq!(
        parse_request(b"nonsense", ParseLimits::default()),
        Err(ParseError::Incomplete)
    );
    assert_eq!(
        parse_request(b"GET / HTTP/2.0\r\n\r\n", ParseLimits::default()),
        Err(ParseError::Version)
    );
    assert_eq!(
        parse_request(
            b"GET / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n",
            ParseLimits::default()
        ),
        Err(ParseError::Chunked)
    );
    let limits = ParseLimits {
        max_body_size: 1,
        ..ParseLimits::default()
    };
    assert_eq!(
        parse_request(b"POST / HTTP/1.1\r\nContent-Length: 2\r\n\r\nxx", limits),
        Err(ParseError::BodyTooLarge)
    );
}

#[test]
fn response_has_framing_and_head_suppresses_body() {
    let response = Response::text(Status::OK, "hello");
    let full = String::from_utf8(response.serialize(false)).unwrap();
    assert!(full.contains("Content-Length: 5\r\n"));
    assert!(full.ends_with("\r\n\r\nhello"));
    assert!(
        String::from_utf8(response.serialize(true))
            .unwrap()
            .ends_with("\r\n\r\n")
    );
}
