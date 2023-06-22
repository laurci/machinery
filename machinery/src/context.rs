use axum::http::HeaderMap;

#[derive(Debug)]
pub struct Context {
    pub headers: HeaderMap,
}
