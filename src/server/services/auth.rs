use std::env;

use axum::{Json, http::StatusCode};
use thiserror::Error;

use crate::server::db::{Db, DbError};


#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing required environment variable {0}")]
    MissingEnv(&'static str),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(DbError),
    #[error("token error: {0}")]
    Token(String),
    #[error("password error: {0}")]
    Password(String),

}

impl From<DbError> for AuthError {
	fn from(value: DbError) -> Self {
		Self::Database(value)
	}
}

struct AuthErrorResponse {
	error: String
}

impl From<AuthError> for (StatusCode, Json<AuthErrorResponse>) {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::MissingEnv(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: message.to_string(),
                }),
            ),
            AuthError::Validation(message) => (
                StatusCode::BAD_REQUEST,
                Json(AuthErrorResponse { error: message }),
            ),
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "Invalid login or password".to_string(),
                }),
            ),
            AuthError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse { error: message }),
            ),
            AuthError::Conflict(message) => (
                StatusCode::CONFLICT,
                Json(AuthErrorResponse { error: message }),
            ),
            AuthError::Database(message) => {
				(
					StatusCode::INTERNAL_SERVER_ERROR,
                	Json(AuthErrorResponse { error: format!("Database error: {} ", message )})
				)
			}
			AuthError::Token(message)
            | AuthError::Password(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse { error: message }),
            ),
        }
    }
}


pub struct AuthService {
	db: Db,
	jwt_secret: String,
    jwt_exp_seconds: u64,
    cookie_secure: bool,
}

impl AuthService {
	pub async  fn from_env(db: Db) -> Result<Self, AuthError> {

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| AuthError::MissingEnv("JWT_SECRET"))?;

        let jwt_exp_seconds = env::var("JWT_EXP_SECONDS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(60 * 60 * 24 * 7);

		let cookie_secure = env::var("AUTH_COOKIE_SECURE").map_err(|_| AuthError::MissingEnv("AUTH_COOKIE_SECURE"))? == "true";

		let out = Self { db, jwt_secret, jwt_exp_seconds, cookie_secure };
		out.run_migrations().await?;

		Ok(out)
	}

	pub async fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.db.connection()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    password_hash TEXT NOT NULL,
                    created_at INTEGER NOT NULL
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_username ON users (username)",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users (
			email)",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        Ok(())
    }

}