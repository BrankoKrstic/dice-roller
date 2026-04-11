use std::env;

use axum::http::{header, HeaderName};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const DEFAULT_LOG_FILTER: &str = "debug,dice_roller=info,tower_http=info";

#[derive(Clone, Copy)]
enum LogFormat {
    Pretty,
    Json,
}

pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(DEFAULT_LOG_FILTER))?;
    let fmt_layer = match log_format() {
        LogFormat::Pretty => tracing_subscriber::fmt::layer()
            .with_target(false)
            .pretty()
            .boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer()
            .with_target(true)
            .json()
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

pub fn sensitive_headers() -> Vec<HeaderName> {
    vec![header::AUTHORIZATION, header::COOKIE]
}

fn log_format() -> LogFormat {
    match env::var("APP_LOG_FORMAT")
        .ok()
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("json") => LogFormat::Json,
        _ => LogFormat::Pretty,
    }
}
