use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;

use crate::client::utils::roll_feed::{DiceRoll, DiceRollFeed};
use crate::shared::utils::time::format_timestamp;

static LOCAL_ROOM_ROLL_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoomRosterEntry {
    pub display_name: String,
    pub presence_note: String,
    pub status_label: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RoomStub {
    pub room_title: String,
    pub room_id: String,
    pub room_note: String,
    pub live_users: Vec<RoomRosterEntry>,
    pub pending_users: Vec<RoomRosterEntry>,
    pub recent_activity_line: String,
    pub activity_feed: DiceRollFeed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoomSummary {
    pub room_title: String,
    pub room_id: String,
    pub room_note: String,
    pub live_users: Vec<RoomRosterEntry>,
    pub pending_users: Vec<RoomRosterEntry>,
    pub recent_activity_line: String,
}

pub(crate) fn room_summaries() -> Vec<RoomSummary> {
    seeded_rooms()
        .into_iter()
        .map(|room| RoomSummary {
            room_title: room.room_title,
            room_id: room.room_id,
            room_note: room.room_note,
            live_users: room.live_users,
            pending_users: room.pending_users,
            recent_activity_line: room.recent_activity_line,
        })
        .collect()
}

pub(crate) fn join_target_from_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(room_route(trimmed))
}

pub(crate) fn room_route(room_id: &str) -> String {
    format!("/room/{}", percent_encode(room_id))
}

pub(crate) fn normalize_room_id_input(room_id: &str) -> String {
    percent_decode(room_id).trim().to_string()
}

pub(crate) fn find_room_by_id(route_room_id: &str) -> Option<RoomStub> {
    let room_id = normalize_room_id_input(route_room_id);

    seeded_rooms()
        .into_iter()
        .find(|room| room.room_id == room_id.as_str())
}

pub(crate) fn build_local_room_roll(expr: &str, result: i64, breakdown: &str) -> DiceRoll {
    let now = Utc::now();
    let sequence = LOCAL_ROOM_ROLL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = now
        .timestamp_nanos_opt()
        .unwrap_or_else(|| now.timestamp_micros() * 1_000);

    DiceRoll {
        id: format!("local-room-roll-{timestamp}-{sequence}"),
        user_id: "local-room-user".to_string(),
        username: "You".to_string(),
        ts: format_timestamp(now),
        expr: expr.to_string(),
        result,
        breakdown: breakdown.to_string(),
    }
}

pub(crate) fn seeded_rooms() -> Vec<RoomStub> {
    vec![
        RoomStub {
            room_title: "Moonlit Ledger".to_string(),
            room_id: "moonlit-ledger".to_string(),
            room_note: "Active table".to_string(),
            live_users: vec![
                roster_entry(
                    "Aria Vale",
                    "Calling initiative for the breach.",
                    Some("GM"),
                ),
                roster_entry(
                    "Tobin Ash",
                    "Waiting on a contested stealth roll.",
                    Some("Live"),
                ),
                roster_entry("Mira Quill", "Tracking the vault ward clock.", None),
            ],
            pending_users: vec![roster_entry(
                "Jun Harrow",
                "Requested approval from the foyer.",
                Some("Pending"),
            )],
            recent_activity_line: "Latest motion: Mira logged a ward pulse at 14 total."
                .to_string(),
            activity_feed: DiceRollFeed {
                rolls: vec![
                    seeded_roll(
                        "room-moonlit-ledger-roll-1",
                        "aria-vale",
                        "Aria Vale",
                        "2026-03-11 19:02:00",
                        "d20 + 4",
                        19,
                        "d20 + 4\n15 + 4 = 19",
                    ),
                    seeded_roll(
                        "room-moonlit-ledger-roll-2",
                        "mira-quill",
                        "Mira Quill",
                        "2026-03-11 19:04:00",
                        "2d6 + 2",
                        14,
                        "2d6 + 2\n[6, 6] + 2 = 14",
                    ),
                ],
                has_more: false,
            },
        },
        RoomStub {
            room_title: "North Gate Audit".to_string(),
            room_id: "north-gate-audit".to_string(),
            room_note: "Quiet table".to_string(),
            live_users: vec![
                roster_entry("Sable Knox", "Reviewing prior scene totals.", Some("Live")),
                roster_entry("Edda Flint", "Holding the action log.", None),
            ],
            pending_users: vec![],
            recent_activity_line: "Latest motion: Sable closed the watch check at 11 total."
                .to_string(),
            activity_feed: DiceRollFeed {
                rolls: vec![seeded_roll(
                    "room-north-gate-audit-roll-1",
                    "sable-knox",
                    "Sable Knox",
                    "2026-03-11 18:56:00",
                    "d20 + 1",
                    11,
                    "d20 + 1\n10 + 1 = 11",
                )],
                has_more: false,
            },
        },
        RoomStub {
            room_title: "Copper Annex".to_string(),
            room_id: "copper-annex".to_string(),
            room_note: "Approval queue open".to_string(),
            live_users: vec![
                roster_entry("Pax Rowan", "Setting the next clue reveal.", Some("Host")),
                roster_entry(
                    "Nell Fenn",
                    "Watching the latest damage spread.",
                    Some("Live"),
                ),
            ],
            pending_users: vec![
                roster_entry("Cato Reef", "Needs approval for seat two.", Some("Pending")),
                roster_entry(
                    "Lio Morrow",
                    "Needs approval for the side bench.",
                    Some("Pending"),
                ),
            ],
            recent_activity_line: "Latest motion: Pax staged a clue check at 17 total.".to_string(),
            activity_feed: DiceRollFeed {
                rolls: vec![seeded_roll(
                    "room-copper-annex-roll-1",
                    "pax-rowan",
                    "Pax Rowan",
                    "2026-03-11 18:59:00",
                    "d20adv + 3",
                    17,
                    "d20adv + 3\n[14, 9] + 3 = 17",
                )],
                has_more: false,
            },
        },
    ]
}

fn roster_entry(
    display_name: &str,
    presence_note: &str,
    status_label: Option<&str>,
) -> RoomRosterEntry {
    RoomRosterEntry {
        display_name: display_name.to_string(),
        presence_note: presence_note.to_string(),
        status_label: status_label.map(str::to_string),
    }
}

fn seeded_roll(
    id: &str,
    user_id: &str,
    username: &str,
    ts: &str,
    expr: &str,
    result: i64,
    breakdown: &str,
) -> DiceRoll {
    DiceRoll {
        id: id.to_string(),
        user_id: user_id.to_string(),
        username: username.to_string(),
        ts: ts.to_string(),
        expr: expr.to_string(),
        result,
        breakdown: breakdown.to_string(),
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

fn percent_decode(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hi = bytes[index + 1] as char;
            let lo = bytes[index + 2] as char;

            if let (Some(hi), Some(lo)) = (hi.to_digit(16), lo.to_digit(16)) {
                decoded.push(((hi << 4) | lo) as u8);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_string())
}
