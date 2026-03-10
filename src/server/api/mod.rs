use axum::{Router, extract::FromRef};
use leptos::config::LeptosOptions;

use crate::server::{api::auth::create_auth_router, services::auth::AuthService};

pub mod auth;
mod roll;
mod router;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub auth: AuthService,
}

pub async fn create_router() -> Router<AppState> {
    Router::new().nest("/auth", create_auth_router())
}
