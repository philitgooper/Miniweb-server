/// Internal errors shared by protocol helpers.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("unsupported method")]
    UnsupportedMethod,
}
