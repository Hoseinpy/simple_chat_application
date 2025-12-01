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
    use super::extract_request_ip;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_request_ip_provide_needed_header() {
        let default_ip = "1.1.1.1";
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_str(&default_ip).unwrap(),
        );

        let result = extract_request_ip(&headers);

        assert_eq!(result, default_ip)
    }
    #[test]
    fn test_extract_request_ip_epmty_header() {
        let result = extract_request_ip(&HeaderMap::new());

        assert_eq!(result, "127.0.0.1")
    }
}
