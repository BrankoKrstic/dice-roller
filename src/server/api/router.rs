use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tracing::{Span, warn};

use crate::{
    server::{
        api::{AppState, presets::create_presets_router, rooms::create_rooms_router},
        services::auth::{AuthError, AuthErrorResponse, AuthService},
    },
    shared::data::user::AuthUser,
};

pub fn create_protected_router(auth: AuthService) -> Router<AppState> {
    Router::new()
        .nest("/presets", create_presets_router())
        .nest("/rooms", create_rooms_router())
        .route_layer(middleware::from_fn_with_state(auth, require_auth))
}

async fn require_auth(
    State(auth): State<AuthService>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    let user = match auth.check_token(jar) {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!(outcome = "missing_auth_cookie", "authentication required");
            return unauthorized_response("Authentication required");
        }
        Err(error) => {
            warn!(error = %error, outcome = "invalid_auth_token", "authentication failed");
            let response: (StatusCode, Json<AuthErrorResponse>) = error.into();
            return response.into_response();
        }
    };

    Span::current().record("user_id", tracing::field::display(user.id.into_inner()));
    request.extensions_mut().insert::<AuthUser>(user);
    next.run(request).await
}

fn unauthorized_response(message: &str) -> Response {
    let error: (StatusCode, Json<AuthErrorResponse>) =
        AuthError::Unauthorized(message.to_string()).into();
    error.into_response()
}
