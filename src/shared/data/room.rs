use serde::{Deserialize, Serialize};

use crate::{
    dsl::{interpreter::EvalResult, parser::Ast},
    shared::data::user::{Email, UserId, Username},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoomId(pub i64);

impl RoomId {
    pub fn into_inner(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoomRollId(pub i64);

impl RoomRollId {
    pub fn into_inner(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InviteRoomMemberRequest {
    pub email: Email,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub creator_id: UserId,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomMembershipStatus {
    Pending,
    Joined,
    Kicked,
}

impl RoomMembershipStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Joined => "joined",
            Self::Kicked => "kicked",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "joined" => Some(Self::Joined),
            "kicked" => Some(Self::Kicked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomMembership {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub status: RoomMembershipStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomRollRequest {
    pub roll_expression: Ast,
    pub roll_result: EvalResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomRoll {
    pub id: RoomRollId,
    pub user_id: UserId,
    pub roll_expression: Ast,
    pub roll_result: EvalResult,
    pub final_result: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRoomMember {
    pub user_id: UserId,
    pub username: Username,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomMemberSummary {
    pub user_id: UserId,
    pub username: Username,
    pub status: RoomMembershipStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomRollSummary {
    pub id: RoomRollId,
    pub user_id: UserId,
    pub username: Username,
    pub roll_expression: Ast,
    pub roll_result: EvalResult,
    pub final_result: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomRollPage {
    pub rolls: Vec<RoomRollSummary>,
    pub next_before_id: Option<RoomRollId>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomStreamSnapshot {
    pub room: Room,
    pub can_manage_members: bool,
    pub active_members: Vec<ActiveRoomMember>,
    pub pending_members: Vec<RoomMemberSummary>,
    pub recent_rolls: RoomRollPage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoomStreamEvent {
    Snapshot {
        snapshot: RoomStreamSnapshot,
    },
    PresenceChanged {
        active_members: Vec<ActiveRoomMember>,
    },
    PendingMembersChanged {
        pending_members: Vec<RoomMemberSummary>,
    },
    RollCreated {
        roll: RoomRollSummary,
    },
    AccessRevoked {
        reason: String,
    },
}
