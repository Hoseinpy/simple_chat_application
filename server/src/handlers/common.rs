use axum::{http::StatusCode, response::IntoResponse};

use crate::models::ApiResponse;

pub async fn handle_version() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    ApiResponse::build(true, version, StatusCode::OK)
}

pub async fn handle_health() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_test_server;

    #[tokio::test]
    async fn test_handle_version() {
        let server = get_test_server().await;

        let response = server.get("/version").await;
        assert_eq!(response.status_code(), 200)
    }
    #[tokio::test]
    async fn test_handle_health() {
        let server = get_test_server().await;

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), 200)
    }
}
