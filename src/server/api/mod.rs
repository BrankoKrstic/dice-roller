use axum::{Router, extract::FromRef};
use leptos::config::LeptosOptions;

use crate::server::{
    api::{auth::create_auth_router, router::create_protected_router},
    services::{auth::AuthService, presets::PresetService},
};

pub mod auth;
pub mod presets;
mod roll;
mod router;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub auth: AuthService,
    pub presets: PresetService,
}

pub fn create_router(auth: AuthService) -> Router<AppState> {
    Router::new()
        .nest("/auth", create_auth_router())
        .merge(create_protected_router(auth))
}
