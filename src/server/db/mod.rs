use std::{env, sync::Arc};

use libsql::{Builder, Connection, Database};
use thiserror::Error;

#[derive(Clone)]
pub struct Db {
    db: Arc<Database>,
}

fn is_remote_database_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("libsql://")
        || normalized.starts_with("https://")
        || normalized.starts_with("http://")
        || normalized.starts_with("wss://")
}

#[derive(Debug, Error)]
pub enum DbError {
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
    Database(String),
}

impl Db {
    pub async fn from_env() -> Result<Self, DbError> {
        let db_url = env::var("TURSO_DATABASE_URL")
            .map_err(|_| DbError::MissingEnv("TURSO_DATABASE_URL"))?;

        let db = if is_remote_database_url(&db_url) {
            let db_token = env::var("TURSO_AUTH_TOKEN")
                .map_err(|_| DbError::MissingEnv("TURSO_AUTH_TOKEN"))?;
            Builder::new_remote(db_url, db_token)
                .build()
                .await
                .map_err(|error| DbError::Database(error.to_string()))?
        } else {
            Builder::new_local(db_url)
                .build()
                .await
                .map_err(|error| DbError::Database(error.to_string()))?
        };

        let db = Self { db: Arc::new(db) };

        Ok(db)
    }
    pub fn connection(&self) -> Result<Connection, DbError> {
        self.db
            .connect()
            .map_err(|error| DbError::Database(error.to_string()))
    }
}
