use axum::{Json, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    server::db::{Db, DbError},
    shared::data::{
        preset::{Preset, PresetId, PresetRequest},
        user::UserId,
    },
};

#[derive(Debug, Error)]
pub enum PresetError {
    #[error("user not found")]
    UserNotFound,
    #[error("preset not found")]
    PresetNotFound,
    #[error("database error: {0}")]
    Database(DbError),
}

impl From<DbError> for PresetError {
    fn from(value: DbError) -> Self {
        Self::Database(value)
    }
}

impl From<libsql::Error> for PresetError {
    fn from(value: libsql::Error) -> Self {
        Self::Database(DbError::Database(value.to_string()))
    }
}

#[derive(Clone, Serialize)]
pub struct PresetErrorResponse {
    error: String,
}

impl From<PresetError> for (StatusCode, Json<PresetErrorResponse>) {
    fn from(value: PresetError) -> Self {
        match value {
            PresetError::UserNotFound => (
                StatusCode::NOT_FOUND,
                Json(PresetErrorResponse {
                    error: "User not found".to_string(),
                }),
            ),
            PresetError::PresetNotFound => (
                StatusCode::NOT_FOUND,
                Json(PresetErrorResponse {
                    error: "Preset not found".to_string(),
                }),
            ),
            PresetError::Database(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PresetErrorResponse {
                    error: format!("Database error: {} ", message),
                }),
            ),
        }
    }
}

#[derive(Clone)]
pub struct PresetService {
    db: Db,
}

impl PresetService {
    pub async fn from_env(db: Db) -> Result<Self, PresetError> {
        let out = Self { db };
        out.run_migrations().await?;

        Ok(out)
    }

    pub async fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.db.connection()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS presets (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    expr TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    archived INTEGER NOT NULL,
                    CONSTRAINT fk_user
                        FOREIGN KEY (user_id)
                        REFERENCES users(id)
                        ON DELETE CASCADE
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_presets_user_id ON presets (user_id)",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        Ok(())
    }

    pub async fn list_presets(&self, user_id: UserId) -> Result<Vec<Preset>, PresetError> {
        let conn = self.db.connection()?;
        let mut rows = conn
            .query(
                "SELECT id, name, expr
                FROM presets
                WHERE user_id = ?1 AND archived = 0
                ORDER BY created_at",
                [user_id.into_inner()],
            )
            .await?;

        let mut presets = Vec::new();
        while let Some(row) = rows.next().await? {
            presets.push(Preset {
                id: PresetId(row.get::<i64>(0)?),
                name: row.get::<String>(1)?,
                expr: row.get::<String>(2)?,
            });
        }

        Ok(presets)
    }

    pub async fn create_preset(
        &self,
        user_id: UserId,
        preset: PresetRequest,
    ) -> Result<Preset, PresetError> {
        let conn = self.db.connection()?;
        let mut rows = conn
            .query(
                "INSERT INTO presets (user_id, name, expr, created_at, updated_at, archived)
                VALUES (?1, ?2, ?3, unixepoch('now'), unixepoch('now'), 0)
                RETURNING id, name, expr",
                (user_id.into_inner(), preset.name, preset.expr),
            )
            .await
            .map_err(map_create_preset_error)?;

        let Some(row) = rows.next().await.map_err(map_create_preset_error)? else {
            return Err(PresetError::Database(DbError::Database(
                "Failed to create preset".to_string(),
            )));
        };

        Ok(Preset {
            id: PresetId(row.get::<i64>(0)?),
            name: row.get::<String>(1)?,
            expr: row.get::<String>(2)?,
        })
    }

    pub async fn delete_preset(
        &self,
        user_id: UserId,
        preset_id: PresetId,
    ) -> Result<(), PresetError> {
        let conn = self.db.connection()?;
        let rows_affected = conn
            .execute(
                "UPDATE presets
                SET archived = 1, updated_at = unixepoch('now')
                WHERE id = ?1 AND user_id = ?2 AND archived = 0",
                (preset_id.0, user_id.into_inner()),
            )
            .await?;

        if rows_affected == 0 {
            return Err(PresetError::PresetNotFound);
        }

        Ok(())
    }
}

fn map_create_preset_error(error: libsql::Error) -> PresetError {
    let message = error.to_string();

    if message.contains("FOREIGN KEY constraint failed") {
        PresetError::UserNotFound
    } else {
        PresetError::Database(DbError::Database(message))
    }
}
