use crate::model::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, Request};
use axum::response::Response;
use futures::StreamExt;
use hyper::StatusCode;
use reqwest::Url;

// Handle the proxy of all HTTP request to the target.
pub async fn handler(
    State(state): State<AppState>,
    mut req: Request<Body>,
) -> Result<Response<String>, StatusCode> {
    let url = req.uri();
    let base_url = &state.config.base_url;

    let target_url = format!("{base_url}{url}");
    *req.uri_mut() = target_url.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let req_method = req.method().clone();
    let host_target: String = Url::parse(base_url).unwrap().host().unwrap().to_string();

    tracing::info!("calling target {target_url}");
    let mut req_header = req.headers().clone();
    req_header.insert("Host", HeaderValue::from_str(&host_target).unwrap());

    // overwrite support headers to avoid server sending Brotli encoded content!:
    req_header.insert(
        "accept-encoding",
        HeaderValue::from_str("br, gzip, deflate").unwrap(),
    );
    let previous_cookie = req_header.remove("cookie"); // we need to use our cookie

    let body = req.into_body();
    let body_data = body_to_data(body).await;

    let request_builder = state
        .http_client
        .request(req_method.clone(), &target_url)
        .headers(req_header.clone())
        .body(body_data.clone());

    let req = request_builder
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = state.http_client.execute(req).await.map_err(|e| {
        tracing::error!("Unable to call target {e}");
        StatusCode::BAD_GATEWAY
    })?;

    into_axum(previous_cookie, response).await
}

async fn body_to_data(body: Body) -> Vec<u8> {
    let mut collected_data = Vec::new();
    let mut body_stream = body.into_data_stream();
    while let Some(chuck) = body_stream.next().await {
        collected_data.extend(chuck.unwrap());
    }
    collected_data
}

/// Convert `reqwest::Response` to the Axum equivalent.
async fn into_axum(
    previous_cookie: Option<HeaderValue>,
    response: reqwest::Response,
) -> Result<axum::http::Response<String>, StatusCode> {
    let response_headers = response.headers().clone();
    let status = response.status();
    let body = &response.text().await.unwrap();

    let mut builder = Response::builder().status(status);

    // Copy over original response headers, skipping the one that can give us trouble
    for header in response_headers {
        if let Some(header_ame) = header.0 {
            let header_name = header_ame.as_str().to_owned();
            if !header_name.eq("transfer-encoding") && !header_name.eq("cookie") {
                builder = builder.header(header_name, header.1.to_str().unwrap());
            }
        }
    }

    if let Some(cookie_header) = previous_cookie {
        builder = builder.header("cookie", cookie_header);
    }
    let result = builder.body(body.to_owned()).map_err(|e| {
        tracing::error!("Error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(result)
}
