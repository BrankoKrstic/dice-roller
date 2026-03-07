use std::env;

use axum::{Json, http::StatusCode};
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode};
use leptos_use::SameSite;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::db::{Db, DbError};
use crate::server::structures::user::{
    Email, ExistingUser, PasswordHashed, User, UserId, Username,
};

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

const AUTH_COOKIE_NAME: &str = "dice_roller_jwt";


impl From<DbError> for AuthError {
    fn from(value: DbError) -> Self {
        Self::Database(value)
    }
}

impl From<libsql::Error> for AuthError {
    fn from(value: libsql::Error) -> Self {
        Self::Database(DbError::Database(value.to_string()))
    }
}

struct AuthErrorResponse {
    error: String,
}

pub struct AuthUser {
    id: UserId,
    username: Username,
    email: Email,
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
            AuthError::Database(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: format!("Database error: {} ", message),
                }),
            ),
            AuthError::Token(message) | AuthError::Password(message) => (
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

pub struct LoginRequest {
    email: Email,
    password: PasswordHashed,
}

impl AuthService {
    pub async fn from_env(db: Db) -> Result<Self, AuthError> {
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| AuthError::MissingEnv("JWT_SECRET"))?;

        let jwt_exp_seconds = env::var("JWT_EXP_SECONDS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(60 * 60 * 24 * 7);

        let cookie_secure = env::var("AUTH_COOKIE_SECURE")
            .map_err(|_| AuthError::MissingEnv("AUTH_COOKIE_SECURE"))?
            == "true";

        let out = Self {
            db,
            jwt_secret,
            jwt_exp_seconds,
            cookie_secure,
        };
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
                    password TEXT NOT NULL,
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

    pub async fn register(&self, payload: User) -> Result<(), AuthError> {
        let conn = self.db.connection()?;

        let result = conn.execute(
            "INSERT INTO users (username, email, password, created_at VALUES (?1, ?2, ?3, unixepoch('now'))",
            (payload.user_name.as_str(), payload.email.as_str(), payload.password.as_str())
        ).await;

        let result = result.map_err(map_insert_error)?;

        Ok(())
    }
    async fn find_user_by_email(&self, email: &Email) -> Result<Option<ExistingUser>, AuthError> {
        let conn = self.db.connection()?;
        let mut rows = conn
            .query(
                "SELECT id, username, email, password FROM users where email = ?1",
                [email.as_str()],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        let id = UserId::new(row.get::<i64>(0)?);
        let username = Username::new(row.get::<String>(1)?);
        let email = Email::new(row.get::<String>(1)?);
        let password = PasswordHashed::new(row.get::<String>(1)?);

        Ok(Some(ExistingUser::new(id, email, username, password)))
    }
    pub async fn login(&self, payload: LoginRequest) -> Result<AuthUser, AuthError> {
        let user = self.find_user_by_email(&payload.email).await?;
        let Some(user) = user else {
            return Err(AuthError::InvalidCredentials);
        };

        user.password
            .verify(payload.password.as_str())
            .map_err(|_| AuthError::InvalidCredentials)?;

        Ok(AuthUser {
            id: user.id,
            username: user.user_name,
            email: user.email,
        })
    }
    pub async fn issue_token(&self, user: AuthUser) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user.id.into_inner(),
            username: user.username.into_inner(),
            email: user.email.into_inner(),
            iat: now,
            exp: now + self.jwt_exp_seconds as i64,
        };

        jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|error| AuthError::Token(error.to_string()))
    }
    pub async fn check_token(&self, token: &str) -> Result<AuthUser, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);

        validation.validate_exp = true;

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|error| AuthError::Unauthorized(error.to_string()))?;

        Ok(AuthUser {
            id: UserId::new(data.claims.sub),
            username: Username::new(data.claims.username),
            email: Email::new(data.claims.email),
        })
    }
    
    pub fn auth_cookie(&self, token: String) -> Cookie<'static> {
        Cookie::build((AUTH_COOKIE_NAME.to_string(), token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.cookie_secure)
            .build()
    }

    pub fn clear_auth_cookie(&self) -> Cookie<'static> {
        Cookie::build((AUTH_COOKIE_NAME.to_string(), String::new()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.cookie_secure)
            .build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    username: String,
    email: String,
    exp: i64,
    iat: i64,
}

fn map_insert_error(error: libsql::Error) -> AuthError {
    let message = error.to_string();

    if message.contains("UNIQUE constraint failed: users.username") {
        AuthError::Conflict("username is already taken".to_string())
    } else if message.contains("UNIQUE constraint failed: users.email") {
        AuthError::Conflict("email is already registered".to_string())
    } else if message.contains("UNIQUE constraint failed") {
        AuthError::Conflict("account already exists".to_string())
    } else {
        AuthError::Database(DbError::Database(message))
    }
}
