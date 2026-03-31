use axum::{Json, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    server::db::{Db, DbError},
    shared::data::{
        room::{
            ActiveRoomMember, CreateRoomRequest, JoinedRoomSummary, Room, RoomId,
            RoomMemberSummary, RoomMembership, RoomMembershipStatus, RoomRoll, RoomRollId,
            RoomRollPage, RoomRollRequest, RoomRollSummary, RoomRosterMember,
            RoomStreamSnapshot, RoomViewerState, RoomViewerStatus,
        },
        user::{Email, UserId, Username},
    },
};

pub const DEFAULT_ROOM_ROLL_PAGE_LIMIT: usize = 50;
pub const MAX_ROOM_ROLL_PAGE_LIMIT: usize = 100;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomReadAccess {
    pub room: Room,
    pub can_manage_members: bool,
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

    pub async fn authorize_room_read(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
    ) -> Result<RoomReadAccess, RoomError> {
        let conn = self.db.connection()?;
        authorize_room_read(&conn, viewer_id, room_id).await
    }

    pub async fn get_room_viewer_state(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
    ) -> Result<RoomViewerState, RoomError> {
        let conn = self.db.connection()?;
        let room = require_room(&conn, room_id).await?;

        if room.creator_id == viewer_id {
            return Ok(RoomViewerState {
                room,
                viewer_status: RoomViewerStatus::Creator,
                can_manage_members: true,
            });
        }

        let Some(membership) = fetch_membership(&conn, room_id, viewer_id).await? else {
            return Err(RoomError::MembershipRequired);
        };

        let viewer_status = match membership.status {
            RoomMembershipStatus::Joined => RoomViewerStatus::Joined,
            RoomMembershipStatus::Pending => RoomViewerStatus::Pending,
            RoomMembershipStatus::Kicked => RoomViewerStatus::Kicked,
        };

        Ok(RoomViewerState {
            room,
            viewer_status,
            can_manage_members: false,
        })
    }

    pub async fn list_joined_rooms(
        &self,
        viewer_id: UserId,
    ) -> Result<Vec<JoinedRoomSummary>, RoomError> {
        let conn = self.db.connection()?;
        fetch_joined_room_summaries(&conn, viewer_id).await
    }

    pub async fn get_room_stream_snapshot(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
        active_members: Vec<ActiveRoomMember>,
        roll_limit: usize,
    ) -> Result<RoomStreamSnapshot, RoomError> {
        let conn = self.db.connection()?;
        let access = authorize_room_read(&conn, viewer_id, room_id).await?;
        let roster_members = fetch_visible_room_roster(&conn, &access, &active_members).await?;
        let recent_rolls =
            fetch_room_roll_page(&conn, room_id, None, clamp_room_roll_page_limit(roll_limit))
                .await?;

        Ok(RoomStreamSnapshot {
            room: access.room,
            can_manage_members: access.can_manage_members,
            roster_members,
            recent_rolls,
        })
    }

    pub async fn get_room_roster_for_reader(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
        active_members: Vec<ActiveRoomMember>,
    ) -> Result<Vec<RoomRosterMember>, RoomError> {
        let conn = self.db.connection()?;
        let access = authorize_room_read(&conn, viewer_id, room_id).await?;
        fetch_visible_room_roster(&conn, &access, &active_members).await
    }

    pub async fn list_managed_members_for_reader(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
    ) -> Result<Vec<RoomMemberSummary>, RoomError> {
        let conn = self.db.connection()?;
        let access = authorize_room_read(&conn, viewer_id, room_id).await?;
        if !access.can_manage_members {
            return Ok(Vec::new());
        }

        fetch_manageable_member_summaries(&conn, room_id, access.room.creator_id).await
    }

    pub async fn list_room_rolls(
        &self,
        viewer_id: UserId,
        room_id: RoomId,
        before_id: Option<RoomRollId>,
        limit: usize,
    ) -> Result<RoomRollPage, RoomError> {
        let conn = self.db.connection()?;
        authorize_room_read(&conn, viewer_id, room_id).await?;

        fetch_room_roll_page(&conn, room_id, before_id, clamp_room_roll_page_limit(limit)).await
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

async fn authorize_room_read(
    conn: &libsql::Connection,
    viewer_id: UserId,
    room_id: RoomId,
) -> Result<RoomReadAccess, RoomError> {
    let room = require_room(conn, room_id).await?;
    if room.creator_id == viewer_id {
        return Ok(RoomReadAccess {
            room,
            can_manage_members: true,
        });
    }

    match fetch_membership(conn, room_id, viewer_id).await? {
        Some(RoomMembership {
            status: RoomMembershipStatus::Joined,
            ..
        }) => Ok(RoomReadAccess {
            room,
            can_manage_members: false,
        }),
        Some(RoomMembership {
            status: RoomMembershipStatus::Pending,
            ..
        }) => Err(RoomError::MembershipPending),
        Some(RoomMembership {
            status: RoomMembershipStatus::Kicked,
            ..
        }) => Err(RoomError::MembershipBlocked),
        None => Err(RoomError::MembershipRequired),
    }
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

async fn fetch_member_summaries_by_status(
    conn: &libsql::Connection,
    room_id: RoomId,
    status: RoomMembershipStatus,
) -> Result<Vec<RoomMemberSummary>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT m.user_id, u.username, m.status
            FROM members m
            JOIN users u ON u.id = m.user_id
            WHERE m.room_id = ?1 AND m.status = ?2
            ORDER BY m.created_at ASC, m.user_id ASC",
            (room_id.into_inner(), status.as_str()),
        )
        .await?;

    let mut members = Vec::new();
    while let Some(row) = rows.next().await? {
        let status = row.get::<String>(2)?;
        let Some(status) = RoomMembershipStatus::from_db(status.as_str()) else {
            return Err(RoomError::Database(DbError::Database(
                "Invalid room membership status in database".to_string(),
            )));
        };

        members.push(RoomMemberSummary {
            user_id: UserId::new(row.get::<i64>(0)?),
            username: Username::new(row.get::<String>(1)?),
            status,
        });
    }

    Ok(members)
}

async fn fetch_manageable_member_summaries(
    conn: &libsql::Connection,
    room_id: RoomId,
    creator_id: UserId,
) -> Result<Vec<RoomMemberSummary>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT m.user_id, u.username, m.status
            FROM members m
            JOIN users u ON u.id = m.user_id
            WHERE m.room_id = ?1 AND m.user_id != ?2
            ORDER BY m.created_at ASC, m.user_id ASC",
            (room_id.into_inner(), creator_id.into_inner()),
        )
        .await?;

    let mut members = Vec::new();
    while let Some(row) = rows.next().await? {
        let status = row.get::<String>(2)?;
        let Some(status) = RoomMembershipStatus::from_db(status.as_str()) else {
            return Err(RoomError::Database(DbError::Database(
                "Invalid room membership status in database".to_string(),
            )));
        };

        members.push(RoomMemberSummary {
            user_id: UserId::new(row.get::<i64>(0)?),
            username: Username::new(row.get::<String>(1)?),
            status,
        });
    }

    Ok(members)
}

async fn fetch_joined_room_summaries(
    conn: &libsql::Connection,
    viewer_id: UserId,
) -> Result<Vec<JoinedRoomSummary>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT DISTINCT r.id, r.creator_id, r.name, r.created_at, r.updated_at
            FROM rooms r
            LEFT JOIN members m
                ON m.room_id = r.id
                AND m.user_id = ?1
            WHERE r.creator_id = ?1 OR m.status = 'joined'
            ORDER BY r.updated_at DESC, r.id DESC",
            [viewer_id.into_inner()],
        )
        .await?;

    let mut rooms = Vec::new();
    while let Some(row) = rows.next().await? {
        let room = Room {
            id: RoomId(row.get::<i64>(0)?),
            creator_id: UserId::new(row.get::<i64>(1)?),
            name: row.get::<String>(2)?,
            created_at: row.get::<i64>(3)?,
            updated_at: row.get::<i64>(4)?,
        };
        let latest_roll = fetch_latest_room_roll_summary(conn, room.id).await?;

        rooms.push(JoinedRoomSummary {
            can_manage_members: room.creator_id == viewer_id,
            room,
            active_member_count: 0,
            latest_roll,
        });
    }

    Ok(rooms)
}

async fn fetch_visible_room_roster(
    conn: &libsql::Connection,
    access: &RoomReadAccess,
    active_members: &[ActiveRoomMember],
) -> Result<Vec<RoomRosterMember>, RoomError> {
    let creator_username = fetch_username(conn, access.room.creator_id).await?;
    let mut roster_members = vec![RoomRosterMember {
        user_id: access.room.creator_id,
        username: creator_username,
        status: RoomMembershipStatus::Joined,
        is_creator: true,
        is_live: active_members
            .iter()
            .any(|member| member.user_id == access.room.creator_id),
    }];

    let members = if access.can_manage_members {
        fetch_manageable_member_summaries(conn, access.room.id, access.room.creator_id).await?
    } else {
        fetch_member_summaries_by_status(conn, access.room.id, RoomMembershipStatus::Joined).await?
    };

    roster_members.extend(members.into_iter().map(|member| RoomRosterMember {
        is_live: active_members
            .iter()
            .any(|active_member| active_member.user_id == member.user_id),
        is_creator: false,
        status: member.status,
        user_id: member.user_id,
        username: member.username,
    }));

    Ok(roster_members)
}

async fn fetch_room_roll_page(
    conn: &libsql::Connection,
    room_id: RoomId,
    before_id: Option<RoomRollId>,
    limit: usize,
) -> Result<RoomRollPage, RoomError> {
    let query_limit = i64::try_from(limit.saturating_add(1)).map_err(|_| {
        RoomError::Database(DbError::Database("invalid roll page size".to_string()))
    })?;
    let mut rows = match before_id {
        Some(before_id) => {
            conn.query(
                "SELECT r.id, r.user_id, u.username, r.roll_expression, r.roll_breakdown, r.final_result, r.created_at, r.updated_at
                FROM room_rolls rr
                JOIN rolls r ON r.id = rr.roll_id
                JOIN users u ON u.id = r.user_id
                WHERE rr.room_id = ?1 AND r.id < ?2
                ORDER BY r.id DESC
                LIMIT ?3",
                (room_id.into_inner(), before_id.into_inner(), query_limit),
            )
            .await?
        }
        None => {
            conn.query(
                "SELECT r.id, r.user_id, u.username, r.roll_expression, r.roll_breakdown, r.final_result, r.created_at, r.updated_at
                FROM room_rolls rr
                JOIN rolls r ON r.id = rr.roll_id
                JOIN users u ON u.id = r.user_id
                WHERE rr.room_id = ?1
                ORDER BY r.id DESC
                LIMIT ?2",
                (room_id.into_inner(), query_limit),
            )
            .await?
        }
    };

    let mut rolls = Vec::new();
    while let Some(row) = rows.next().await? {
        rolls.push(parse_room_roll_summary_row(&row)?);
    }

    let has_more = rolls.len() > limit;
    if has_more {
        rolls.truncate(limit);
    }

    let next_before_id = if has_more {
        rolls.last().map(|roll| roll.id)
    } else {
        None
    };

    Ok(RoomRollPage {
        rolls,
        next_before_id,
        has_more,
    })
}

async fn fetch_username(conn: &libsql::Connection, user_id: UserId) -> Result<Username, RoomError> {
    let mut rows = conn
        .query(
            "SELECT username FROM users WHERE id = ?1",
            [user_id.into_inner()],
        )
        .await?;

    let Some(row) = rows.next().await? else {
        return Err(RoomError::UserNotFound);
    };

    Ok(Username::new(row.get::<String>(0)?))
}

async fn fetch_latest_room_roll_summary(
    conn: &libsql::Connection,
    room_id: RoomId,
) -> Result<Option<RoomRollSummary>, RoomError> {
    let mut rows = conn
        .query(
            "SELECT r.id, r.user_id, u.username, r.roll_expression, r.roll_breakdown, r.final_result, r.created_at, r.updated_at
            FROM room_rolls rr
            JOIN rolls r ON r.id = rr.roll_id
            JOIN users u ON u.id = r.user_id
            WHERE rr.room_id = ?1
            ORDER BY r.id DESC
            LIMIT 1",
            [room_id.into_inner()],
        )
        .await?;

    let Some(row) = rows.next().await? else {
        return Ok(None);
    };

    parse_room_roll_summary_row(&row).map(Some)
}

fn parse_room_roll_summary_row(row: &libsql::Row) -> Result<RoomRollSummary, RoomError> {
    let roll_expression = serde_json::from_str::<crate::dsl::parser::Ast>(&row.get::<String>(3)?)
        .map_err(|error| RoomError::InvalidRollExpression(error.to_string()))?;
    let roll_result =
        serde_json::from_str::<crate::dsl::interpreter::EvalResult>(&row.get::<String>(4)?)
            .map_err(|error| RoomError::InvalidRollResult(error.to_string()))?;

    Ok(RoomRollSummary {
        id: RoomRollId(row.get::<i64>(0)?),
        user_id: UserId::new(row.get::<i64>(1)?),
        username: Username::new(row.get::<String>(2)?),
        roll_expression,
        roll_result,
        final_result: row.get::<i64>(5)?,
        created_at: row.get::<i64>(6)?,
        updated_at: row.get::<i64>(7)?,
    })
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

fn clamp_room_roll_page_limit(limit: usize) -> usize {
    match limit {
        0 => DEFAULT_ROOM_ROLL_PAGE_LIMIT,
        value => value.min(MAX_ROOM_ROLL_PAGE_LIMIT),
    }
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
