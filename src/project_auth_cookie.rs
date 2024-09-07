use crate::error::Error;
use crate::model::AppState;
use axum::extract::State;
use axum::http::{Request, StatusCode};

use reqwest::cookie::CookieStore;
use reqwest::Url;
use reqwest::{cookie::Jar, redirect::Policy, Client};
use std::{collections::HashMap, sync::Arc};

// Intercept all request and make sure all has a valid cookie inside.
// if the project_auth is not preset, it performs the auth.
pub async fn auth_layer<B>(
    State(state): State<AppState>,
    request: Request<B>,
) -> Result<Request<B>, StatusCode> {
    tracing::debug!("Processing Auth...");

    let base_url = Url::parse(&state.config.base_url).unwrap();

    let cookies = state.cookie_jar.cookies(&base_url);

    // skip if it's already pressent
    if let Some(cookie_header) = cookies {
        if let Ok(project_auth) = cookie_header.to_str() {
            if project_auth.contains("project_auth") {
                tracing::info!(" ✅project_auth cookie found!, nothing to do");
                return Ok(request);
            }
        }
    }

    tracing::warn!("project_auth cookie missing");
    let auth_cookie = process_login(&state.config.base_url, &state.config.password).await?;

    tracing::info!(auth_cookie, "Cookie Stored!");
    state.cookie_jar.add_cookie_str(&auth_cookie, &base_url);

    Ok(request)
}

pub async fn process_login(base_url: &str, password: &str) -> Result<String, StatusCode> {
    let password_url = format!("{base_url}/password");

    let cookie_jar = Arc::new(Jar::default());

    let client_for_password = Client::builder()
        .brotli(true)
        .gzip(true)
        .redirect(Policy::none())
        .cookie_store(true)
        .cookie_provider(Arc::clone(&cookie_jar))
        .build()
        .map_err(|e| {
            tracing::warn!("unable to build client {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut form = HashMap::new();
    form.insert("password", password);

    let response = client_for_password
        .post(&password_url)
        .form(&form)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Unable to process login {e}");
            StatusCode::UNAUTHORIZED
        })?;

    response
        .headers()
        .get("set-cookie")
        .map(|c| c.to_str().unwrap().to_owned())
        .ok_or_else(|| {
            tracing::error!("cookie is missing after auth process");
            StatusCode::FORBIDDEN
        })
}
