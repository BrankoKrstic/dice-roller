use futures::StreamExt;
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
    #[error("Database error {0}")]
    DbError(DbError),
}

impl From<DbError> for PresetError {
    fn from(error: DbError) -> PresetError {
        PresetError::DbError(error)
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
                    name TEXT NOT NULL
                    expr TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    archived INTEGER NOT NULL
                    UNIQUE (user_id, name),
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
        let rows = conn.query(
            "SELECT (id, preset_name, expr) FROM presets WHERE user_id = ?1 ORDER BY created_at",
            &[user_id.into_inner()],
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        let out = rows
            .into_stream()
            .map(|row| {
                let row = row.unwrap();

                Preset {
                    id: PresetId(row.get(0).unwrap()),
                    name: row.get(1).unwrap(),
                    expr: row.get(2).unwrap(),
                }
            })
            .collect()
            .await;
        Ok(out)
    }
    pub async fn create_preset(
        &self,
        user_id: UserId,
        preset: PresetRequest,
    ) -> Result<Preset, PresetError> {
        let conn = self.db.connection()?;

        let mut rows = conn
            .query(
                "INSERT (user_id, name, expr, created_at, updated_at, archived)
                INTO presets
                VALUES (?1, ?2, ?3, unixepoch('now'), unixepoch('now'), 0)
                RETURNING (id, preset_name, expr)",
                (user_id.into_inner(), preset.name, preset.expr),
            )
            .await
            .map_err(|error| DbError::Database(error.to_string()))?;
        let row = rows.next().await.unwrap().unwrap();

        Ok(Preset {
            id: PresetId(row.get(0).unwrap()),
            name: row.get(1).unwrap(),
            expr: row.get(2).unwrap(),
        })
    }
}
