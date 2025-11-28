use axum::http::HeaderMap;

pub fn extract_request_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next_back().map(|s| s.trim().to_string()))
        .unwrap_or("127.0.0.1".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_request_ip_successfully() {
        let mut headers = HeaderMap::new();
        let default_ip = "1.1.1.1";
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_str(default_ip).unwrap(),
        );
        let ip = extract_request_ip(&headers);

        assert_eq!(ip, default_ip)
    }
    #[test]
    fn test_extract_request_ip_bad_headers() {
        let headers = HeaderMap::new();
        let ip = extract_request_ip(&headers);

        assert_eq!(ip, "127.0.0.1")
    }
}
