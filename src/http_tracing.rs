use axum::body::Body;

/// Basic logging for all incoming request.
pub async fn log_requests(
    req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    tracing::info!(method = %req.method(), uri = %req.uri(), "-> Received request");
    let response = next.run(req).await;
    tracing::info!(status = %response.status(), "Sent response");
    response
}
