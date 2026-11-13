use super::Method;
use std::collections::HashMap;

/// Case-insensitive HTTP header collection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderMap(HashMap<String, String>);
impl HeaderMap {
    /// Inserts a header, normalizing its name to lowercase.
    pub fn insert(&mut self, name: String, value: String) {
        self.0.insert(name.to_ascii_lowercase(), value);
    }
    /// Finds a header without regard to name case.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(&name.to_ascii_lowercase()).map(String::as_str)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
/// Parsed HTTP request. The target and body are owned to outlive the connection buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub target: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub version: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
impl Request {
    /// Returns a decoded query parameter, if present.
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }
}
