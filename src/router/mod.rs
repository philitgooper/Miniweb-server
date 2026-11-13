use crate::http::{Method, Request, Response, Status};
use std::collections::HashMap;
/// Uniform handler signature.
pub type Handler = fn(Request) -> Response;
/// Simple exact-path router.
#[derive(Default)]
pub struct Router {
    routes: HashMap<String, HashMap<Method, Handler>>,
}
impl Router {
    /// Registers a route.
    pub fn add(&mut self, method: Method, path: &str, handler: Handler) {
        self.routes
            .entry(path.into())
            .or_default()
            .insert(method, handler);
    }
    /// Registers a GET route.
    pub fn get(&mut self, path: &str, handler: Handler) {
        self.add(Method::Get, path, handler)
    }
    /// Dispatches a request or returns 404/405.
    pub fn dispatch(&self, request: Request) -> Response {
        if let Some(methods) = self.routes.get(&request.path) {
            if let Some(h) = methods.get(&request.method) {
                return h(request);
            }
            let mut r = Response::text(Status::METHOD_NOT_ALLOWED, "Method Not Allowed");
            r.headers.insert(
                "Allow".into(),
                methods
                    .keys()
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            return r;
        }
        Response::text(Status::NOT_FOUND, "Not Found")
    }
}
