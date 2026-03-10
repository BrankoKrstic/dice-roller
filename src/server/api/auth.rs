use axum::{Json, Router, extract::State, routing::{get, post}};
use axum_extra::extract::CookieJar;
use axum::http::{StatusCode};

use crate::{server::{api::AppState, services::auth::{AuthErrorResponse, AuthService}, structures::user::{LoginRequest, User}}, shared::data::user::AuthUser};

pub type AuthApiResult<T> = Result<T, (StatusCode, Json<AuthErrorResponse>)>;

#[axum::debug_handler]
async fn user_info_handler(State(auth): State<AuthService>, jar: CookieJar) -> AuthApiResult<Json<Option<AuthUser>>> {
    let user = auth.check_token(jar)?;
    Ok(Json(user))
}

#[axum::debug_handler]
async fn register_handler(
    State(auth): State<AuthService>,
    jar: CookieJar,
    Json(payload): Json<User>,
) -> AuthApiResult<(CookieJar, Json<AuthUser>)> {
    let user = auth.register(payload).await?;
	let token = auth.issue_token(user.clone()).await?;
	let jar = jar.add(auth.auth_cookie(token));

    Ok((jar, Json(user)))
}

#[axum::debug_handler]
async fn login_handler(
    State(auth): State<AuthService>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> AuthApiResult<(CookieJar, Json<AuthUser>)> {
    let user = auth.login(payload).await?;
    let token = auth.issue_token(user.clone()).await?;

    let jar = jar.add(auth.auth_cookie(token));
    Ok((jar, Json(user)))
}

#[axum::debug_handler]
async fn logout_handler(
    State(auth): State<AuthService>,
    jar: CookieJar,
) -> AuthApiResult<CookieJar> {

    let jar = auth.clear_auth_cookie(jar);
    Ok(jar)
}



pub fn create_auth_router() -> Router<AppState> {
    Router::new()
        .route("/me", get(user_info_handler))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
		
}
