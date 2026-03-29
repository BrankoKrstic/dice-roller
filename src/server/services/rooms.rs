use axum::{Json, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    server::db::{Db, DbError},
    shared::data::{
        room::{
            CreateRoomRequest, Room, RoomId, RoomMembership, RoomMembershipStatus, RoomRoll,
            RoomRollId, RoomRollRequest,
        },
        user::{Email, UserId},
    },
};

#[derive(Debug, Error)]
pub enum RoomError {
    #[error("invalid room name")]
    InvalidRoomName,
    #[error("user not found")]
    UserNotFound,
    #[error("room not found")]
    RoomNotFound,
    #[error("membership not found")]
    MembershipNotFound,
    #[error("room creator privileges required")]
    NotRoomCreator,
    #[error("room creator cannot be kicked")]
    CannotKickCreator,
    #[error("membership is already pending")]
    MembershipAlreadyPending,
    #[error("membership is already joined")]
    MembershipAlreadyJoined,
    #[error("membership is already kicked")]
    MembershipAlreadyKicked,
    #[error("membership is blocked")]
    MembershipBlocked,
    #[error("membership is pending approval")]
    MembershipPending,
    #[error("joined membership required")]
    MembershipRequired,
    #[error("invited user not found")]
    InvitedUserNotFound,
    #[error("invalid roll expression: {0}")]
    InvalidRollExpression(String),
    #[error("invalid roll result: {0}")]
    InvalidRollResult(String),
    #[error("database error: {0}")]
    Database(DbError),
}

impl From<DbError> for RoomError {
    fn from(value: DbError) -> Self {
        Self::Database(value)
    }
}

impl From<libsql::Error> for RoomError {
    fn from(value: libsql::Error) -> Self {
        Self::Database(DbError::Database(value.to_string()))
    }
}

#[derive(Clone, Serialize)]
pub struct RoomErrorResponse {
    error: String,
}

impl From<RoomError> for (StatusCode, Json<RoomErrorResponse>) {
    fn from(value: RoomError) -> Self {
        match value {
            RoomError::InvalidRoomName
            | RoomError::CannotKickCreator
            | RoomError::InvalidRollExpression(_)
            | RoomError::InvalidRollResult(_) => (
                StatusCode::BAD_REQUEST,
                Json(RoomErrorResponse {
                    error: value.to_string(),
                }),
            ),
            RoomError::UserNotFound
            | RoomError::RoomNotFound
            | RoomError::MembershipNotFound
            | RoomError::InvitedUserNotFound => (
                StatusCode::NOT_FOUND,
                Json(RoomErrorResponse {
                    error: value.to_string(),
                }),
            ),
            RoomError::NotRoomCreator
            | RoomError::MembershipBlocked
            | RoomError::MembershipPending
            | RoomError::MembershipRequired => (
                StatusCode::FORBIDDEN,
                Json(RoomErrorResponse {
                    error: value.to_string(),
                }),
            ),
            RoomError::MembershipAlreadyPending
            | RoomError::MembershipAlreadyJoined
            | RoomError::MembershipAlreadyKicked => (
                StatusCode::CONFLICT,
                Json(RoomErrorResponse {
                    error: value.to_string(),
                }),
            ),
            RoomError::Database(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RoomErrorResponse {
                    error: format!("Database error: {} ", message),
                }),
            ),
        }
    }
}

#[derive(Clone)]
pub struct RoomService {
    db: Db,
}

impl RoomService {
    pub async fn from_env(db: Db) -> Result<Self, RoomError> {
        let out = Self { db };
        out.run_migrations().await?;

        Ok(out)
    }

    pub async fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.db.connection()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rooms (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    creator_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    CONSTRAINT fk_room_creator
                        FOREIGN KEY (creator_id)
                        REFERENCES users(id)
                        ON DELETE CASCADE
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS members (
                    room_id INTEGER NOT NULL,
                    user_id INTEGER NOT NULL,
                    status TEXT NOT NULL CHECK (status IN ('pending', 'joined', 'kicked')),
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    PRIMARY KEY (room_id, user_id),
                    CONSTRAINT fk_member_room
                        FOREIGN KEY (room_id)
                        REFERENCES rooms(id)
                        ON DELETE CASCADE,
                    CONSTRAINT fk_member_user
                        FOREIGN KEY (user_id)
                        REFERENCES users(id)
                        ON DELETE CASCADE
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rolls (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL,
                    roll_expression TEXT NOT NULL,
                    roll_breakdown TEXT NOT NULL,
                    final_result INTEGER NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    CONSTRAINT fk_roll_user
                        FOREIGN KEY (user_id)
                        REFERENCES users(id)
                        ON DELETE CASCADE
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS room_rolls (
                    room_id INTEGER NOT NULL,
                    roll_id INTEGER NOT NULL,
                    PRIMARY KEY (room_id, roll_id),
                    CONSTRAINT fk_room_roll_room
                        FOREIGN KEY (room_id)
                        REFERENCES rooms(id)
                        ON DELETE CASCADE,
                    CONSTRAINT fk_room_roll_roll
                        FOREIGN KEY (roll_id)
                        REFERENCES rolls(id)
                        ON DELETE CASCADE
                )",
            (),
        )
        .await
        .map_err(|error| DbError::Database(error.to_string()))?;

        for statement in [
            "CREATE INDEX IF NOT EXISTS idx_rooms_creator_id ON rooms (creator_id)",
            "CREATE INDEX IF NOT EXISTS idx_members_room_id ON members (room_id)",
            "CREATE INDEX IF NOT EXISTS idx_members_user_id ON members (user_id)",
            "CREATE INDEX IF NOT EXISTS idx_members_status ON members (status)",
            "CREATE INDEX IF NOT EXISTS idx_rolls_user_id ON rolls (user_id)",
            "CREATE INDEX IF NOT EXISTS idx_room_rolls_roll_id ON room_rolls (roll_id)",
        ] {
            conn.execute(statement, ())
                .await
                .map_err(|error| DbError::Database(error.to_string()))?;
        }

        Ok(())
    }

    pub async fn create_room(
        &self,
        creator_id: UserId,
        payload: CreateRoomRequest,
    ) -> Result<Room, RoomError> {
        let name = validate_room_name(payload.name)?;
        let conn = self.db.connection()?;

        let tx = conn.transaction().await?;
        let room = insert_room(&tx, creator_id, &name).await?;
        insert_membership(&tx, room.id, creator_id, RoomMembershipStatus::Joined).await?;
        tx.commit().await?;

        Ok(room)
    }

    pub async fn request_to_join(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<RoomMembership, RoomError> {
        let conn = self.db.connection()?;
        let room = require_room(&conn, room_id).await?;

        if room.creator_id == user_id {
            return Err(RoomError::MembershipAlreadyJoined);
        }

        if let Some(membership) = fetch_membership(&conn, room_id, user_id).await? {
            return match membership.status {
                RoomMembershipStatus::Pending => Err(RoomError::MembershipAlreadyPending),
                RoomMembershipStatus::Joined => Err(RoomError::MembershipAlreadyJoined),
                RoomMembershipStatus::Kicked => Err(RoomError::MembershipBlocked),
            };
        }

        insert_membership(&conn, room_id, user_id, RoomMembershipStatus::Pending).await
    }

    pub async fn allow_member(
        &self,
        actor_id: UserId,
        room_id: RoomId,
        target_user_id: UserId,
    ) -> Result<RoomMembership, RoomError> {
        let conn = self.db.connection()?;
        require_room_creator(&conn, actor_id, room_id).await?;

        let Some(membership) = fetch_membership(&conn, room_id, target_user_id).await? else {
            return Err(RoomError::MembershipNotFound);
        };

        match membership.status {
            RoomMembershipStatus::Joined => Err(RoomError::MembershipAlreadyJoined),
            RoomMembershipStatus::Pending | RoomMembershipStatus::Kicked => {
                update_membership_status(
                    &conn,
                    room_id,
                    target_user_id,
                    RoomMembershipStatus::Joined,
                )
                .await
            }
        }
    }

    pub async fn kick_member(
        &self,
        actor_id: UserId,
        room_id: RoomId,
        target_user_id: UserId,
    ) -> Result<RoomMembership, RoomError> {
        let conn = self.db.connection()?;
        let room = require_room_creator(&conn, actor_id, room_id).await?;

        if room.creator_id == target_user_id {
            return Err(RoomError::CannotKickCreator);
        }

        let Some(membership) = fetch_membership(&conn, room_id, target_user_id).await? else {
            return Err(RoomError::MembershipNotFound);
        };

        match membership.status {
            RoomMembershipStatus::Kicked => Err(RoomError::MembershipAlreadyKicked),
            RoomMembershipStatus::Pending | RoomMembershipStatus::Joined => {
                update_membership_status(
                    &conn,
                    room_id,
                    target_user_id,
                    RoomMembershipStatus::Kicked,
                )
                .await
            }
        }
    }

    pub async fn add_member_by_email(
        &self,
        actor_id: UserId,
        room_id: RoomId,
        email: Email,
    ) -> Result<RoomMembership, RoomError> {
        let conn = self.db.connection()?;
        let room = require_room_creator(&conn, actor_id, room_id).await?;
        let Some(target_user_id) = find_user_id_by_email(&conn, &email).await? else {
            return Err(RoomError::InvitedUserNotFound);
        };

        if target_user_id == room.creator_id {
            return Err(RoomError::MembershipAlreadyJoined);
        }

        match fetch_membership(&conn, room_id, target_user_id).await? {
            Some(membership) => match membership.status {
                RoomMembershipStatus::Joined => Err(RoomError::MembershipAlreadyJoined),
                RoomMembershipStatus::Pending | RoomMembershipStatus::Kicked => {
                    update_membership_status(
                        &conn,
                        room_id,
                        target_user_id,
                        RoomMembershipStatus::Joined,
                    )
                    .await
                }
            },
            None => {
                insert_membership(&conn, room_id, target_user_id, RoomMembershipStatus::Joined)
                    .await
            }
        }
    }

    pub async fn add_roll_to_room(
        &self,
        user_id: UserId,
        room_id: RoomId,
        payload: RoomRollRequest,
    ) -> Result<RoomRoll, RoomError> {
        let conn = self.db.connection()?;
        let room = require_room(&conn, room_id).await?;

        if room.creator_id != user_id {
            match fetch_membership(&conn, room_id, user_id).await? {
                Some(RoomMembership {
                    status: RoomMembershipStatus::Joined,
                    ..
                }) => {}
                Some(RoomMembership {
                    status: RoomMembershipStatus::Pending,
                    ..
                }) => return Err(RoomError::MembershipPending),
                Some(RoomMembership {
                    status: RoomMembershipStatus::Kicked,
                    ..
                }) => return Err(RoomError::MembershipBlocked),
                None => return Err(RoomError::MembershipRequired),
            }
        }

        let RoomRollRequest {
            roll_expression,
            roll_result,
        } = payload;
        let final_result = roll_result.total();

        let roll_expression_json = serde_json::to_string(&roll_expression)
            .map_err(|error| RoomError::InvalidRollExpression(error.to_string()))?;
        let roll_breakdown_json = serde_json::to_string(&roll_result)
            .map_err(|error| RoomError::InvalidRollResult(error.to_string()))?;

        let tx = conn.transaction().await?;
        let roll = {
            let mut rows = tx
                .query(
                    "INSERT INTO rolls (
                        user_id,
                        roll_expression,
                        roll_breakdown,
                        final_result,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, unixepoch('now'), unixepoch('now'))
                    RETURNING id, created_at, updated_at",
                    (
                        user_id.into_inner(),
                        roll_expression_json,
                        roll_breakdown_json,
                        final_result,
                    ),
                )
                .await
                .map_err(map_roll_insert_error)?;

            let Some(row) = rows.next().await.map_err(map_roll_insert_error)? else {
                return Err(RoomError::Database(DbError::Database(
                    "Failed to create room roll".to_string(),
                )));
            };

            RoomRoll {
                id: RoomRollId(row.get::<i64>(0)?),
                user_id,
                roll_expression,
                roll_result,
                final_result,
                created_at: row.get::<i64>(1)?,
                updated_at: row.get::<i64>(2)?,
            }
        };

        tx.execute(
            "INSERT INTO room_rolls (room_id, roll_id) VALUES (?1, ?2)",
            (room_id.into_inner(), roll.id.into_inner()),
        )
        .await
        .map_err(map_room_roll_link_error)?;
        tx.commit().await?;

        Ok(roll)
    }
}

async fn require_room(conn: &libsql::Connection, room_id: RoomId) -> Result<Room, RoomError> {
    fetch_room(conn, room_id)
        .await?
        .ok_or(RoomError::RoomNotFound)
}

async fn require_room_creator(
    conn: &libsql::Connection,
    actor_id: UserId,
    room_id: RoomId,
) -> Result<Room, RoomError> {
    let room = require_room(conn, room_id).await?;
    if room.creator_id != actor_id {
        return Err(RoomError::NotRoomCreator);
    }

    Ok(room)
}

async fn fetch_room(conn: &libsql::Connection, room_id: RoomId) -> Result<Option<Room>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT id, creator_id, name, created_at, updated_at
            FROM rooms
            WHERE id = ?1",
            [room_id.into_inner()],
        )
        .await?;

    let Some(row) = rows.next().await? else {
        return Ok(None);
    };

    Ok(Some(Room {
        id: RoomId(row.get::<i64>(0)?),
        creator_id: UserId::new(row.get::<i64>(1)?),
        name: row.get::<String>(2)?,
        created_at: row.get::<i64>(3)?,
        updated_at: row.get::<i64>(4)?,
    }))
}

async fn fetch_membership(
    conn: &libsql::Connection,
    room_id: RoomId,
    user_id: UserId,
) -> Result<Option<RoomMembership>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT room_id, user_id, status, created_at, updated_at
            FROM members
            WHERE room_id = ?1 AND user_id = ?2",
            (room_id.into_inner(), user_id.into_inner()),
        )
        .await?;

    let Some(row) = rows.next().await? else {
        return Ok(None);
    };

    let status = row.get::<String>(2)?;
    let Some(status) = RoomMembershipStatus::from_db(status.as_str()) else {
        return Err(RoomError::Database(DbError::Database(
            "Invalid room membership status in database".to_string(),
        )));
    };

    Ok(Some(RoomMembership {
        room_id: RoomId(row.get::<i64>(0)?),
        user_id: UserId::new(row.get::<i64>(1)?),
        status,
        created_at: row.get::<i64>(3)?,
        updated_at: row.get::<i64>(4)?,
    }))
}

async fn find_user_id_by_email(
    conn: &libsql::Connection,
    email: &Email,
) -> Result<Option<UserId>, RoomError> {
    let mut rows = conn
        .query("SELECT id FROM users WHERE email = ?1", [email.as_str()])
        .await?;

    let Some(row) = rows.next().await? else {
        return Ok(None);
    };

    Ok(Some(UserId::new(row.get::<i64>(0)?)))
}

async fn insert_room(
    conn: &libsql::Connection,
    creator_id: UserId,
    name: &str,
) -> Result<Room, RoomError> {
    let mut rows = conn
        .query(
            "INSERT INTO rooms (creator_id, name, created_at, updated_at)
            VALUES (?1, ?2, unixepoch('now'), unixepoch('now'))
            RETURNING id, creator_id, name, created_at, updated_at",
            (creator_id.into_inner(), name),
        )
        .await
        .map_err(map_room_insert_error)?;

    let Some(row) = rows.next().await.map_err(map_room_insert_error)? else {
        return Err(RoomError::Database(DbError::Database(
            "Failed to create room".to_string(),
        )));
    };

    Ok(Room {
        id: RoomId(row.get::<i64>(0)?),
        creator_id: UserId::new(row.get::<i64>(1)?),
        name: row.get::<String>(2)?,
        created_at: row.get::<i64>(3)?,
        updated_at: row.get::<i64>(4)?,
    })
}

async fn insert_membership(
    conn: &libsql::Connection,
    room_id: RoomId,
    user_id: UserId,
    status: RoomMembershipStatus,
) -> Result<RoomMembership, RoomError> {
    let mut rows = conn
        .query(
            "INSERT INTO members (room_id, user_id, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, unixepoch('now'), unixepoch('now'))
            RETURNING room_id, user_id, status, created_at, updated_at",
            (room_id.into_inner(), user_id.into_inner(), status.as_str()),
        )
        .await
        .map_err(map_membership_insert_error)?;

    let Some(row) = rows.next().await.map_err(map_membership_insert_error)? else {
        return Err(RoomError::Database(DbError::Database(
            "Failed to create membership".to_string(),
        )));
    };

    Ok(RoomMembership {
        room_id: RoomId(row.get::<i64>(0)?),
        user_id: UserId::new(row.get::<i64>(1)?),
        status,
        created_at: row.get::<i64>(3)?,
        updated_at: row.get::<i64>(4)?,
    })
}

async fn update_membership_status(
    conn: &libsql::Connection,
    room_id: RoomId,
    user_id: UserId,
    status: RoomMembershipStatus,
) -> Result<RoomMembership, RoomError> {
    let mut rows = conn
        .query(
            "UPDATE members
            SET status = ?3, updated_at = unixepoch('now')
            WHERE room_id = ?1 AND user_id = ?2
            RETURNING room_id, user_id, status, created_at, updated_at",
            (room_id.into_inner(), user_id.into_inner(), status.as_str()),
        )
        .await?;

    let Some(row) = rows.next().await? else {
        return Err(RoomError::MembershipNotFound);
    };

    Ok(RoomMembership {
        room_id: RoomId(row.get::<i64>(0)?),
        user_id: UserId::new(row.get::<i64>(1)?),
        status,
        created_at: row.get::<i64>(3)?,
        updated_at: row.get::<i64>(4)?,
    })
}

fn validate_room_name(name: String) -> Result<String, RoomError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RoomError::InvalidRoomName);
    }

    Ok(trimmed.to_string())
}

fn map_room_insert_error(error: libsql::Error) -> RoomError {
    let message = error.to_string();

    if message.contains("FOREIGN KEY constraint failed") {
        RoomError::UserNotFound
    } else {
        RoomError::Database(DbError::Database(message))
    }
}

fn map_membership_insert_error(error: libsql::Error) -> RoomError {
    let message = error.to_string();

    if message.contains("UNIQUE constraint failed") {
        RoomError::MembershipAlreadyJoined
    } else if message.contains("FOREIGN KEY constraint failed") {
        RoomError::UserNotFound
    } else {
        RoomError::Database(DbError::Database(message))
    }
}

fn map_roll_insert_error(error: libsql::Error) -> RoomError {
    let message = error.to_string();

    if message.contains("FOREIGN KEY constraint failed") {
        RoomError::UserNotFound
    } else {
        RoomError::Database(DbError::Database(message))
    }
}

fn map_room_roll_link_error(error: libsql::Error) -> RoomError {
    let message = error.to_string();

    if message.contains("FOREIGN KEY constraint failed") {
        RoomError::RoomNotFound
    } else {
        RoomError::Database(DbError::Database(message))
    }
}
