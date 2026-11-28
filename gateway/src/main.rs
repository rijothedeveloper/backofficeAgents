use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use tokio::net::TcpListener;

fn app() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body, json!({"status": "ok"}));
    }

    #[tokio::test]
    async fn invalid_route_returns_404() {
        let request = Request::builder().uri("/nope").body(Body::empty()).unwrap();

        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
