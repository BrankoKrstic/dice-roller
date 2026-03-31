use chrono::{TimeZone, Utc};

#[cfg(feature = "hydrate")]
use crate::shared::data::room::{RoomId, RoomStreamEvent};
use crate::{
    client::utils::{
        api::parse_error_response,
        roll_feed::{DiceRoll, DiceRollFeed},
        url::base_url,
    },
    shared::{
        data::room::{
            ActiveRoomMember, CreateRoomRequest, JoinedRoomSummary, Room, RoomMembership, RoomRoll,
            RoomRollId, RoomRollPage, RoomRollRequest, RoomRollSummary, RoomViewerState,
        },
        utils::time::format_timestamp,
    },
};

pub fn room_route(room_id: i64) -> String {
    format!("/room/{room_id}")
}

pub fn parse_room_id_input(input: &str) -> Result<i64, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Enter a room ID.".to_string());
    }

    trimmed
        .parse::<i64>()
        .map_err(|_| "Room IDs use digits only.".to_string())
}

pub fn active_member_count_label(count: usize) -> String {
    if count == 1 {
        "1 live in room".to_string()
    } else {
        format!("{count} live in room")
    }
}

pub fn active_member_preview(active_members: &[ActiveRoomMember]) -> String {
    let names = active_members
        .iter()
        .map(|member| member.username.as_str())
        .collect::<Vec<_>>();

    match names.as_slice() {
        [] => "Nobody is seated yet.".to_string(),
        [one] => format!("{one} is already at the table."),
        [one, two] => format!("{one} and {two} are already at the table."),
        [one, two, rest @ ..] => {
            format!(
                "{one}, {two}, and {} more are already at the table.",
                rest.len()
            )
        }
    }
}

pub fn latest_roll_activity_line(latest_roll: &Option<RoomRollSummary>) -> String {
    latest_roll
        .as_ref()
        .map(|roll| {
            format!(
                "Latest motion: {} logged {} at {} total.",
                roll.username.as_str(),
                roll.roll_expression,
                roll.final_result
            )
        })
        .unwrap_or_else(|| "Latest motion: The ledger is quiet for now.".to_string())
}

pub fn room_roll_feed_from_page(page: &RoomRollPage) -> DiceRollFeed {
    DiceRollFeed {
        rolls: page
            .rolls
            .iter()
            .rev()
            .map(room_roll_summary_to_dice_roll)
            .collect(),
        has_more: page.has_more,
    }
}

pub fn prepend_room_roll_page(feed: &mut DiceRollFeed, page: &RoomRollPage) {
    let mut older_rolls = page
        .rolls
        .iter()
        .rev()
        .map(room_roll_summary_to_dice_roll)
        .collect::<Vec<_>>();
    older_rolls.extend(feed.rolls.clone());
    feed.rolls = older_rolls;
    feed.has_more = page.has_more;
}

pub fn append_live_room_roll(feed: &mut DiceRollFeed, roll: &RoomRollSummary) {
    let next_roll = room_roll_summary_to_dice_roll(roll);
    if feed
        .rolls
        .iter()
        .any(|existing| existing.id == next_roll.id)
    {
        return;
    }

    feed.rolls.push(next_roll);
}

pub fn room_roll_summary_to_dice_roll(roll: &RoomRollSummary) -> DiceRoll {
    DiceRoll {
        id: roll.id.into_inner().to_string(),
        user_id: roll.user_id.into_inner().to_string(),
        username: roll.username.as_str().to_string(),
        ts: unix_timestamp_label(roll.created_at),
        expr: roll.roll_expression.to_string(),
        result: roll.final_result,
        breakdown: roll.roll_result.to_string(),
    }
}

pub fn room_roll_before_id(feed: &DiceRollFeed) -> Option<String> {
    feed.rolls.first().map(|roll| roll.id.clone())
}

pub async fn list_joined_rooms_request() -> Result<Vec<JoinedRoomSummary>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/rooms", base_url()))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to load rooms").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn create_room_request(name: String) -> Result<Room, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/rooms", base_url()))
        .json(&CreateRoomRequest { name })
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to create room").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn join_room_request(room_id: i64) -> Result<RoomMembership, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/rooms/{room_id}/join", base_url()))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to join room").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn get_room_access_request(room_id: i64) -> Result<RoomViewerState, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/rooms/{room_id}/access", base_url()))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to load room").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn allow_member_request(room_id: i64, user_id: i64) -> Result<RoomMembership, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/api/rooms/{room_id}/members/{user_id}/allow",
            base_url()
        ))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to update room member").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn kick_member_request(room_id: i64, user_id: i64) -> Result<RoomMembership, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/api/rooms/{room_id}/members/{user_id}/kick",
            base_url()
        ))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to kick room member").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn list_room_rolls_request(
    room_id: i64,
    before_id: Option<RoomRollId>,
) -> Result<RoomRollPage, String> {
    let client = reqwest::Client::new();
    let url = before_id.map_or_else(
        || format!("{}/api/rooms/{room_id}/rolls", base_url()),
        |before_id| {
            format!(
                "{}/api/rooms/{room_id}/rolls?before_id={}",
                base_url(),
                before_id.into_inner()
            )
        },
    );

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to load room activity").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

pub async fn add_room_roll_request(room_id: i64, expression: &str) -> Result<RoomRoll, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/rooms/{room_id}/rolls", base_url()))
        .json(&RoomRollRequest {
            expression: expression.to_string(),
        })
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to submit roll").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

fn unix_timestamp_label(timestamp: i64) -> String {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(format_timestamp)
        .unwrap_or_else(|| timestamp.to_string())
}

#[cfg(feature = "hydrate")]
pub struct RoomEventStream {
    source: web_sys::EventSource,
    _snapshot: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _roster: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _roll_created: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _access_revoked: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    _open: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}

#[cfg(feature = "hydrate")]
impl RoomEventStream {
    pub fn connect(
        room_id: RoomId,
        on_event: impl Fn(RoomStreamEvent) + 'static,
        on_connection_change: impl Fn(bool) + 'static,
        on_error: impl Fn(String) + 'static,
    ) -> Result<Self, String> {
        use wasm_bindgen::{JsCast, closure::Closure};

        let source = web_sys::EventSource::new(&format!(
            "{}/api/rooms/{}/events",
            base_url(),
            room_id.into_inner()
        ))
        .map_err(|error| format!("{error:?}"))?;

        let on_event = std::rc::Rc::new(on_event);
        let on_error = std::rc::Rc::new(on_error);
        let on_connection_change = std::rc::Rc::new(on_connection_change);

        let snapshot = room_stream_listener("snapshot", on_event.clone(), on_error.clone());
        let roster = room_stream_listener("roster_changed", on_event.clone(), on_error.clone());
        let roll_created = room_stream_listener("roll_created", on_event.clone(), on_error.clone());
        let access_revoked =
            room_stream_listener("access_revoked", on_event.clone(), on_error.clone());

        source
            .add_event_listener_with_callback("snapshot", snapshot.as_ref().unchecked_ref())
            .map_err(|error| format!("{error:?}"))?;
        source
            .add_event_listener_with_callback("roster_changed", roster.as_ref().unchecked_ref())
            .map_err(|error| format!("{error:?}"))?;
        source
            .add_event_listener_with_callback("roll_created", roll_created.as_ref().unchecked_ref())
            .map_err(|error| format!("{error:?}"))?;
        source
            .add_event_listener_with_callback(
                "access_revoked",
                access_revoked.as_ref().unchecked_ref(),
            )
            .map_err(|error| format!("{error:?}"))?;

        let open_on_connection_change = on_connection_change.clone();
        let open = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            open_on_connection_change(true);
        }) as Box<dyn FnMut(_)>);
        source.set_onopen(Some(open.as_ref().unchecked_ref()));

        let error_on_connection_change = on_connection_change;
        let error_on_error = on_error;
        let error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            error_on_connection_change(false);
            error_on_error("Live room connection is retrying.".to_string());
        }) as Box<dyn FnMut(_)>);
        source.set_onerror(Some(error.as_ref().unchecked_ref()));

        Ok(Self {
            source,
            _snapshot: snapshot,
            _roster: roster,
            _roll_created: roll_created,
            _access_revoked: access_revoked,
            _error: error,
            _open: open,
        })
    }
}

#[cfg(feature = "hydrate")]
impl Drop for RoomEventStream {
    fn drop(&mut self) {
        self.source.close();
    }
}

#[cfg(feature = "hydrate")]
pub struct IntervalHandle {
    id: i32,
    _callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
}

#[cfg(feature = "hydrate")]
impl IntervalHandle {
    pub fn start(interval_ms: i32, callback: impl FnMut() + 'static) -> Result<Self, String> {
        use wasm_bindgen::{JsCast, closure::Closure};

        let window = web_sys::window().ok_or_else(|| "Missing browser window".to_string())?;
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                interval_ms,
            )
            .map_err(|error| format!("{error:?}"))?;
        Ok(Self {
            id,
            _callback: callback,
        })
    }
}

#[cfg(feature = "hydrate")]
impl Drop for IntervalHandle {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.id);
        }
    }
}

#[cfg(feature = "hydrate")]
fn room_stream_listener(
    event_name: &str,
    on_event: std::rc::Rc<dyn Fn(RoomStreamEvent)>,
    on_error: std::rc::Rc<dyn Fn(String)>,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)> {
    use wasm_bindgen::closure::Closure;

    let event_name = event_name.to_string();
    Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let payload = event
            .data()
            .as_string()
            .ok_or_else(|| format!("Missing {event_name} payload"))
            .and_then(|text| {
                serde_json::from_str::<RoomStreamEvent>(&text)
                    .map_err(|error| format!("Invalid {event_name} event: {error}"))
            });

        match payload {
            Ok(event) => on_event(event),
            Err(message) => on_error(message),
        }
    }) as Box<dyn FnMut(_)>)
}
