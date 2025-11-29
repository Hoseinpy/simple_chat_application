use axum::http::HeaderMap;

pub fn extract_request_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next_back().map(|s| s.trim().to_string()))
        .unwrap_or("127.0.0.1".to_string())
}
