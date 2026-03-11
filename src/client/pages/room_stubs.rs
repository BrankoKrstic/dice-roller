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

pub(crate) fn find_room_by_id(route_room_id: &str) -> Option<RoomStub> {
    let decoded = percent_decode(route_room_id);
    let room_id = decoded.trim();

    seeded_rooms()
        .into_iter()
        .find(|room| room.room_id == room_id)
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
                        "2m ago",
                        "d20 + 4",
                        19,
                        "d20 + 4\n15 + 4 = 19",
                    ),
                    seeded_roll(
                        "room-moonlit-ledger-roll-2",
                        "mira-quill",
                        "Mira Quill",
                        "just now",
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
                    "8m ago",
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
                    "5m ago",
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

#[cfg(test)]
mod tests {
    use super::{build_local_room_roll, find_room_by_id, join_target_from_input, seeded_rooms};

    #[test]
    fn join_target_is_none_for_blank_input() {
        assert_eq!(join_target_from_input("   "), None);
    }

    #[test]
    fn join_target_url_encodes_trimmed_room_id() {
        assert_eq!(
            join_target_from_input("  Table 7/West Wing  "),
            Some("/room/Table%207%2FWest%20Wing".to_string())
        );
    }

    #[test]
    fn find_room_by_id_matches_seeded_room() {
        let room = find_room_by_id("  moonlit-ledger  ").expect("expected seeded room");

        assert_eq!(room.room_id, "moonlit-ledger");
        assert_eq!(room.room_title, "Moonlit Ledger");
    }

    #[test]
    fn find_room_by_id_returns_none_for_unknown_room() {
        assert!(find_room_by_id("unknown-room").is_none());
    }

    #[test]
    fn find_room_by_id_matches_canonical_seeded_room_id() {
        let room = find_room_by_id("  copper-annex  ").expect("expected seeded room");

        assert_eq!(room.room_id, "copper-annex");
        assert_eq!(room.room_title, "Copper Annex");
    }

    #[test]
    fn appended_room_roll_has_unique_id_and_local_metadata() {
        let first_roll = build_local_room_roll("2d6 + 4", 11, "2d6 + 4\n[3, 4] + 4 = 11");
        let second_roll = build_local_room_roll("d20 + 2", 17, "d20 + 2\n15 + 2 = 17");

        assert_ne!(first_roll.id, second_roll.id);
        assert!(first_roll.id.starts_with("local-room-roll-"));
        assert_eq!(first_roll.user_id, "local-room-user");
        assert_eq!(first_roll.username, "You");
        assert_eq!(first_roll.expr, "2d6 + 4");
        assert_eq!(first_roll.result, 11);
        assert_eq!(first_roll.breakdown, "2d6 + 4\n[3, 4] + 4 = 11");
        assert!(!first_roll.ts.is_empty());
        assert!(first_roll.ts.contains('-'));
        assert!(first_roll.ts.contains(':'));
    }

    #[test]
    fn appending_room_roll_does_not_mutate_seeded_room_feed() {
        let seeded_room = seeded_rooms()
            .into_iter()
            .find(|room| room.room_id == "moonlit-ledger")
            .expect("expected seeded room");
        let seeded_roll_count = seeded_room.activity_feed.rolls.len();

        let mut local_feed = seeded_room.activity_feed.clone();
        let appended_roll = build_local_room_roll("3d6", 12, "3d6\n[4, 4, 4] = 12");
        let appended_roll_id = appended_roll.id.clone();
        local_feed.add_roll(appended_roll);

        assert_eq!(local_feed.rolls.len(), seeded_roll_count + 1);
        assert_eq!(local_feed.rolls.last().expect("expected appended roll").id, appended_roll_id);
        assert_eq!(
            local_feed.rolls.first().expect("expected seeded roll").id,
            "room-moonlit-ledger-roll-1"
        );

        let refreshed_seeded_room = seeded_rooms()
            .into_iter()
            .find(|room| room.room_id == "moonlit-ledger")
            .expect("expected seeded room");

        assert_eq!(refreshed_seeded_room.activity_feed.rolls.len(), seeded_roll_count);
        assert_eq!(
            refreshed_seeded_room
                .activity_feed
                .rolls
                .last()
                .expect("expected seeded roll")
                .id,
            "room-moonlit-ledger-roll-2"
        );
    }
}
