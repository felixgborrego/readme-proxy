use crate::error::Result;
use reqwest::{cookie::Jar, Client};
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub cookie_jar: Arc<Jar>,
    pub config: Arc<Config>,
    pub http_client: Client,
}
pub struct Config {
    pub base_url: String,
    pub password: String,
}

impl AppState {
    pub fn from_env() -> Result<Self> {
        let mut base_url = env::var("BASE_URL").expect("BASE_URL is missing");
        let password = env::var("PASSWORD").expect("PASSWORD is missing");

        tracing::info!(base_url, "Env config");
        if base_url.ends_with('/') {
            base_url.pop();
        }

        let cookie_jar = Arc::new(Jar::default());
        let http_client = Client::builder()
            .brotli(true)
            .gzip(true)
            .cookie_store(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()?;

        let config = Arc::new(Config { base_url, password });

        Ok(Self {
            cookie_jar,
            config,
            http_client,
        })
    }
}
