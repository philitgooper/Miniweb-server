use miniweb_server::{
    http::{ParseLimits, Request, Response, Status, parse_request},
    router::Router,
};
fn ok(_: Request) -> Response {
    Response::text(Status::OK, "ok")
}
#[test]
fn routes_and_reports_404_or_405() {
    let mut router = Router::default();
    router.get("/x", ok);
    let get = parse_request(b"GET /x HTTP/1.1\r\n\r\n", ParseLimits::default()).unwrap();
    assert_eq!(router.dispatch(get).status.code, 200);
    let post = parse_request(b"POST /x HTTP/1.1\r\n\r\n", ParseLimits::default()).unwrap();
    let response = router.dispatch(post);
    assert_eq!(response.status.code, 405);
    assert_eq!(
        response.headers.get("Allow").map(String::as_str),
        Some("GET")
    );
    let unknown = parse_request(b"GET /missing HTTP/1.1\r\n\r\n", ParseLimits::default()).unwrap();
    assert_eq!(router.dispatch(unknown).status.code, 404);
}
