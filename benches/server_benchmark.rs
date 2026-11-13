use criterion::{Criterion, black_box, criterion_group, criterion_main};
use miniweb_server::{
    http::{ParseLimits, Request, Response, Status, parse_request},
    router::Router,
};

fn ok(_: Request) -> Response {
    Response::text(Status::OK, "ok")
}

fn protocol(c: &mut Criterion) {
    let bytes = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    c.bench_function("parse_simple_get", |b| {
        b.iter(|| parse_request(black_box(bytes), ParseLimits::default()))
    });
    let request = match parse_request(bytes, ParseLimits::default()) {
        Ok(request) => request,
        Err(_) => return,
    };
    let mut router = Router::default();
    router.get("/", ok);
    c.bench_function("route_simple_get", |b| {
        b.iter(|| router.dispatch(black_box(request.clone())))
    });
    let response = Response::text(Status::OK, "hello");
    c.bench_function("serialize_response", |b| {
        b.iter(|| response.serialize(false))
    });
}

criterion_group!(benches, protocol);
criterion_main!(benches);
