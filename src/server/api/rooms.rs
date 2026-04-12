use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures::{
    StreamExt,
    stream::{self},
};
use serde::Deserialize;
use tokio::sync::{broadcast, watch};
use tracing::info;

use crate::{
    server::{
        api::AppState,
        services::rooms::{
            DEFAULT_ROOM_ROLL_PAGE_LIMIT, RoomError, RoomErrorResponse, RoomService,
        },
    },
    shared::data::{
        room::{
            ActiveRoomMember, AddRoomMemberRequest, CreateRoomRequest, JoinedRoomSummary, Room,
            RoomId, RoomMembership, RoomRoll, RoomRollId, RoomRollPage, RoomRollRequest,
            RoomRollSummary, RoomStreamEvent, RoomViewerState,
        },
        user::{AuthUser, UserId},
    },
};

pub type RoomApiResult<T> = Result<T, (StatusCode, Json<RoomErrorResponse>)>;

#[derive(Clone, Default)]
pub struct RoomLiveHub {
    inner: Arc<Mutex<RoomLiveHubState>>,
}

struct RoomLiveHubState {
    rooms: HashMap<RoomId, LiveRoomState>,
}

struct LiveRoomState {
    next_connection_id: u64,
    broadcaster: broadcast::Sender<RoomHubEvent>,
    connections: HashMap<u64, LiveConnection>,
    active_members: BTreeMap<UserId, ActiveMemberState>,
}

#[derive(Clone)]
struct ActiveMemberState {
    member: ActiveRoomMember,
    connection_count: usize,
}

struct LiveConnection {
    user_id: UserId,
    shutdown: watch::Sender<Option<String>>,
}

#[derive(Clone)]
enum RoomHubEvent {
    RosterChanged,
    RollCreated { roll: RoomRollSummary },
}

struct RoomLiveSubscription {
    connection_id: u64,
    room_events: broadcast::Receiver<RoomHubEvent>,
    shutdown: watch::Receiver<Option<String>>,
    active_members: Vec<ActiveRoomMember>,
}

struct RoomLiveConnectionGuard {
    room_id: RoomId,
    connection_id: Option<u64>,
    room_live: RoomLiveHub,
}

impl RoomLiveConnectionGuard {
    fn new(room_live: RoomLiveHub, room_id: RoomId, connection_id: u64) -> Self {
        Self {
            room_id,
            connection_id: Some(connection_id),
            room_live,
        }
    }
}

impl Drop for RoomLiveConnectionGuard {
    fn drop(&mut self) {
        if let Some(connection_id) = self.connection_id.take() {
            self.room_live.unsubscribe(self.room_id, connection_id);
        }
    }
}

impl Default for RoomLiveHubState {
    fn default() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

impl RoomLiveHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn subscribe(&self, room_id: RoomId, user: &AuthUser) -> RoomLiveSubscription {
        let (sender, active_members, connection_id, room_events, shutdown) = {
            let mut state = self.inner.lock().expect("room live hub lock poisoned");
            let room = state
                .rooms
                .entry(room_id)
                .or_insert_with(new_live_room_state);
            let connection_id = room.next_connection_id;
            room.next_connection_id += 1;

            let room_events = room.broadcaster.subscribe();
            let (shutdown_tx, shutdown_rx) = watch::channel(None::<String>);
            room.connections.insert(
                connection_id,
                LiveConnection {
                    user_id: user.id,
                    shutdown: shutdown_tx,
                },
            );

            room.active_members
                .entry(user.id)
                .and_modify(|entry| entry.connection_count += 1)
                .or_insert_with(|| ActiveMemberState {
                    member: ActiveRoomMember {
                        user_id: user.id,
                        username: user.username.clone(),
                    },
                    connection_count: 1,
                });

            (
                room.broadcaster.clone(),
                collect_active_members(room),
                connection_id,
                room_events,
                shutdown_rx,
            )
        };

        let _ = sender.send(RoomHubEvent::RosterChanged);

        RoomLiveSubscription {
            connection_id,
            room_events,
            shutdown,
            active_members,
        }
    }

    fn unsubscribe(&self, room_id: RoomId, connection_id: u64) {
        let notification = {
            let mut state = self.inner.lock().expect("room live hub lock poisoned");
            let Some(room) = state.rooms.get_mut(&room_id) else {
                return;
            };
            let Some(connection) = room.connections.remove(&connection_id) else {
                return;
            };

            decrement_active_member(room, connection.user_id);
            let notification = if room.connections.is_empty() {
                None
            } else {
                Some(room.broadcaster.clone())
            };

            if room.connections.is_empty() {
                state.rooms.remove(&room_id);
            }

            notification
        };

        if let Some(sender) = notification {
            let _ = sender.send(RoomHubEvent::RosterChanged);
        }
    }

    fn active_members(&self, room_id: RoomId) -> Vec<ActiveRoomMember> {
        let state = self.inner.lock().expect("room live hub lock poisoned");
        state
            .rooms
            .get(&room_id)
            .map(collect_active_members)
            .unwrap_or_default()
    }

    fn notify_roster_changed(&self, room_id: RoomId) {
        let sender = {
            let state = self.inner.lock().expect("room live hub lock poisoned");
            state
                .rooms
                .get(&room_id)
                .map(|room| room.broadcaster.clone())
        };

        if let Some(sender) = sender {
            let _ = sender.send(RoomHubEvent::RosterChanged);
        }
    }

    fn notify_roll_created(&self, room_id: RoomId, roll: RoomRollSummary) {
        let sender = {
            let state = self.inner.lock().expect("room live hub lock poisoned");
            state
                .rooms
                .get(&room_id)
                .map(|room| room.broadcaster.clone())
        };

        if let Some(sender) = sender {
            let _ = sender.send(RoomHubEvent::RollCreated { roll });
        }
    }

    fn revoke_user(&self, room_id: RoomId, user_id: UserId, reason: String) {
        let (shutdowns, notification) = {
            let mut state = self.inner.lock().expect("room live hub lock poisoned");
            let Some(room) = state.rooms.get_mut(&room_id) else {
                return;
            };

            let connection_ids = room
                .connections
                .iter()
                .filter_map(|(connection_id, connection)| {
                    (connection.user_id == user_id).then_some(*connection_id)
                })
                .collect::<Vec<_>>();

            if connection_ids.is_empty() {
                return;
            }

            let mut shutdowns = Vec::new();
            for connection_id in connection_ids {
                if let Some(connection) = room.connections.remove(&connection_id) {
                    shutdowns.push(connection.shutdown);
                }
            }

            room.active_members.remove(&user_id);
            let notification = if room.connections.is_empty() {
                None
            } else {
                Some(room.broadcaster.clone())
            };

            if room.connections.is_empty() {
                state.rooms.remove(&room_id);
            }

            (shutdowns, notification)
        };

        for shutdown in shutdowns {
            let _ = shutdown.send(Some(reason.clone()));
        }

        if let Some(sender) = notification {
            let _ = sender.send(RoomHubEvent::RosterChanged);
        }
    }

    fn revoke_room(&self, room_id: RoomId, reason: String) {
        let shutdowns = {
            let mut state = self.inner.lock().expect("room live hub lock poisoned");
            let Some(room) = state.rooms.remove(&room_id) else {
                return;
            };

            room.connections
                .into_values()
                .map(|connection| connection.shutdown)
                .collect::<Vec<_>>()
        };

        for shutdown in shutdowns {
            let _ = shutdown.send(Some(reason.clone()));
        }
    }
}

#[derive(Deserialize)]
struct RoomRollsQuery {
    before_id: Option<RoomRollId>,
    limit: Option<usize>,
}

struct RoomStreamState {
    room_id: RoomId,
    viewer_id: UserId,
    done: bool,
    rooms: RoomService,
    room_live: RoomLiveHub,
    room_events: broadcast::Receiver<RoomHubEvent>,
    shutdown: watch::Receiver<Option<String>>,
    _subscription_guard: RoomLiveConnectionGuard,
}

#[axum::debug_handler]
async fn create_room_handler(
    State(rooms): State<RoomService>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<CreateRoomRequest>,
) -> RoomApiResult<Json<Room>> {
    let room = rooms.create_room(user.id, payload).await?;
    info!(
        user_id = user.id.into_inner(),
        room_id = room.id.into_inner(),
        "created room"
    );
    Ok(Json(room))
}

#[axum::debug_handler(state = AppState)]
async fn list_rooms_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
) -> RoomApiResult<Json<Vec<JoinedRoomSummary>>> {
    let mut rooms = rooms.list_joined_rooms(user.id).await?;
    for summary in &mut rooms {
        summary.active_member_count = room_live.active_members(summary.room.id).len();
    }

    info!(
        user_id = user.id.into_inner(),
        room_count = rooms.len(),
        "listed rooms"
    );
    Ok(Json(rooms))
}

#[axum::debug_handler]
async fn room_access_handler(
    State(rooms): State<RoomService>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
) -> RoomApiResult<Json<RoomViewerState>> {
    let access = rooms.get_room_viewer_state(user.id, room_id).await?;
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        viewer_status = ?access.viewer_status,
        "loaded room access"
    );
    Ok(Json(access))
}

#[axum::debug_handler(state = AppState)]
async fn request_to_join_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
) -> RoomApiResult<Json<RoomMembership>> {
    let membership = rooms.request_to_join(user.id, room_id).await?;
    room_live.notify_roster_changed(room_id);
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        status = ?membership.status,
        "requested room join"
    );

    Ok(Json(membership))
}

#[axum::debug_handler(state = AppState)]
async fn add_room_member_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
    Json(payload): Json<AddRoomMemberRequest>,
) -> RoomApiResult<Json<RoomMembership>> {
    let membership = rooms.add_member(user.id, room_id, payload).await?;
    room_live.notify_roster_changed(room_id);
    info!(
        actor_user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        target_user_id = membership.user_id.into_inner(),
        status = ?membership.status,
        "added room member"
    );

    Ok(Json(membership))
}

#[axum::debug_handler(state = AppState)]
async fn allow_member_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path((room_id, target_user_id)): Path<(RoomId, UserId)>,
) -> RoomApiResult<Json<RoomMembership>> {
    let membership = rooms.allow_member(user.id, room_id, target_user_id).await?;
    room_live.notify_roster_changed(room_id);
    info!(
        actor_user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        target_user_id = target_user_id.into_inner(),
        status = ?membership.status,
        "allowed room member"
    );

    Ok(Json(membership))
}

#[axum::debug_handler(state = AppState)]
async fn kick_member_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path((room_id, target_user_id)): Path<(RoomId, UserId)>,
) -> RoomApiResult<Json<RoomMembership>> {
    let membership = rooms.kick_member(user.id, room_id, target_user_id).await?;
    room_live.revoke_user(
        room_id,
        target_user_id,
        "room membership revoked".to_string(),
    );
    room_live.notify_roster_changed(room_id);
    info!(
        actor_user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        target_user_id = target_user_id.into_inner(),
        status = ?membership.status,
        "kicked room member"
    );

    Ok(Json(membership))
}

#[axum::debug_handler(state = AppState)]
async fn leave_room_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
) -> RoomApiResult<Json<RoomMembership>> {
    let membership = rooms.leave_room(user.id, room_id).await?;
    room_live.revoke_user(room_id, user.id, "room access changed".to_string());
    room_live.notify_roster_changed(room_id);
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        status = ?membership.status,
        "left room"
    );

    Ok(Json(membership))
}

#[axum::debug_handler(state = AppState)]
async fn archive_room_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
) -> RoomApiResult<Json<Room>> {
    let room = rooms.archive_room(user.id, room_id).await?;
    room_live.revoke_room(room_id, "room was archived".to_string());
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        "archived room"
    );

    Ok(Json(room))
}

#[axum::debug_handler(state = AppState)]
async fn add_room_roll_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
    Json(payload): Json<RoomRollRequest>,
) -> RoomApiResult<Json<RoomRoll>> {
    let roll = rooms.add_roll_to_room(user.id, room_id, payload).await?;
    room_live.notify_roll_created(room_id, room_roll_summary_from_roll(&user, &roll));
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        roll_id = roll.id.into_inner(),
        final_result = roll.final_result,
        "created room roll"
    );

    Ok(Json(roll))
}

#[axum::debug_handler]
async fn list_room_rolls_handler(
    State(rooms): State<RoomService>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
    Query(query): Query<RoomRollsQuery>,
) -> RoomApiResult<Json<RoomRollPage>> {
    let rolls = rooms
        .list_room_rolls(
            user.id,
            room_id,
            query.before_id,
            query.limit.unwrap_or(DEFAULT_ROOM_ROLL_PAGE_LIMIT),
        )
        .await?;

    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        roll_count = rolls.rolls.len(),
        has_more = rolls.has_more,
        "listed room rolls"
    );
    Ok(Json(rolls))
}

#[axum::debug_handler(state = AppState)]
async fn room_events_handler(
    State(rooms): State<RoomService>,
    State(room_live): State<RoomLiveHub>,
    Extension(user): Extension<AuthUser>,
    Path(room_id): Path<RoomId>,
) -> RoomApiResult<Response> {
    rooms.authorize_room_read(user.id, room_id).await?;
    let subscription = room_live.subscribe(room_id, &user);
    let subscription_guard =
        RoomLiveConnectionGuard::new(room_live.clone(), room_id, subscription.connection_id);
    let snapshot = match rooms
        .get_room_stream_snapshot(
            user.id,
            room_id,
            subscription.active_members.clone(),
            DEFAULT_ROOM_ROLL_PAGE_LIMIT,
        )
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => return Err(error.into()),
    };
    info!(
        user_id = user.id.into_inner(),
        room_id = room_id.into_inner(),
        "opened room event stream"
    );

    let initial_event = RoomStreamEvent::Snapshot { snapshot };
    let initial = stream::once(async move { Ok::<Event, Infallible>(sse_event(&initial_event)) });
    let updates = stream::unfold(
        RoomStreamState {
            room_id,
            viewer_id: user.id,
            done: false,
            rooms,
            room_live: room_live.clone(),
            room_events: subscription.room_events,
            shutdown: subscription.shutdown,
            _subscription_guard: subscription_guard,
        },
        |mut state| async move {
            if state.done {
                return None;
            }

            loop {
                tokio::select! {
                    changed = state.shutdown.changed() => {
                        if changed.is_err() {
                            return None;
                        }

                        let Some(reason) = state.shutdown.borrow().clone() else {
                            continue;
                        };

                        state.done = true;
                        let event = RoomStreamEvent::AccessRevoked { reason };
                        return Some((Ok(sse_event(&event)), state));
                    }
                    result = state.room_events.recv() => {
                        match result {
                            Ok(RoomHubEvent::RosterChanged) => {
                                let roster_members = match state
                                    .rooms
                                    .get_room_roster_for_reader(
                                        state.viewer_id,
                                        state.room_id,
                                        state.room_live.active_members(state.room_id),
                                    )
                                    .await
                                {
                                    Ok(roster_members) => roster_members,
                                    Err(
                                        RoomError::MembershipBlocked
                                        | RoomError::MembershipLeft
                                        | RoomError::MembershipPending
                                        | RoomError::MembershipRequired
                                        | RoomError::RoomArchived,
                                    ) => {
                                        state.done = true;
                                        let event = RoomStreamEvent::AccessRevoked {
                                            reason: "room access changed".to_string(),
                                        };
                                        return Some((Ok(sse_event(&event)), state));
                                    }
                                    Err(_) => {
                                        return None;
                                    }
                                };

                                let event = RoomStreamEvent::RosterChanged { roster_members };
                                return Some((Ok(sse_event(&event)), state));
                            }
                            Ok(RoomHubEvent::RollCreated { roll }) => {
                                let event = RoomStreamEvent::RollCreated { roll };
                                return Some((Ok(sse_event(&event)), state));
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                let snapshot = match state
                                    .rooms
                                    .get_room_stream_snapshot(
                                        state.viewer_id,
                                        state.room_id,
                                        state.room_live.active_members(state.room_id),
                                        DEFAULT_ROOM_ROLL_PAGE_LIMIT,
                                    )
                                    .await
                                {
                                    Ok(snapshot) => snapshot,
                                    Err(
                                        RoomError::MembershipBlocked
                                        | RoomError::MembershipLeft
                                        | RoomError::MembershipPending
                                        | RoomError::MembershipRequired
                                        | RoomError::RoomArchived,
                                    ) => {
                                        state.done = true;
                                        let event = RoomStreamEvent::AccessRevoked {
                                            reason: "room access changed".to_string(),
                                        };
                                        return Some((Ok(sse_event(&event)), state));
                                    }
                                    Err(_) => {
                                        return None;
                                    }
                                };

                                let event = RoomStreamEvent::Snapshot { snapshot };
                                return Some((Ok(sse_event(&event)), state));
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                return None;
                            }
                        }
                    }
                }
            }
        },
    );
    let stream = initial.chain(updates).boxed();

    Ok(Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
        .into_response())
}

pub fn create_rooms_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rooms_handler).post(create_room_handler))
        .route("/{room_id}/archive", post(archive_room_handler))
        .route("/{room_id}/access", get(room_access_handler))
        .route("/{room_id}/join", post(request_to_join_handler))
        .route("/{room_id}/leave", post(leave_room_handler))
        .route("/{room_id}/members", post(add_room_member_handler))
        .route(
            "/{room_id}/members/{user_id}/allow",
            post(allow_member_handler),
        )
        .route(
            "/{room_id}/members/{user_id}/kick",
            post(kick_member_handler),
        )
        .route(
            "/{room_id}/rolls",
            get(list_room_rolls_handler).post(add_room_roll_handler),
        )
        .route("/{room_id}/events", get(room_events_handler))
}

fn new_live_room_state() -> LiveRoomState {
    let (broadcaster, _) = broadcast::channel(64);
    LiveRoomState {
        next_connection_id: 1,
        broadcaster,
        connections: HashMap::new(),
        active_members: BTreeMap::new(),
    }
}

fn collect_active_members(room: &LiveRoomState) -> Vec<ActiveRoomMember> {
    room.active_members
        .values()
        .map(|entry| entry.member.clone())
        .collect()
}

fn decrement_active_member(room: &mut LiveRoomState, user_id: UserId) {
    let should_remove = room
        .active_members
        .get_mut(&user_id)
        .map(|entry| {
            if entry.connection_count > 1 {
                entry.connection_count -= 1;
                false
            } else {
                true
            }
        })
        .unwrap_or(false);

    if should_remove {
        room.active_members.remove(&user_id);
    }
}

fn room_roll_summary_from_roll(user: &AuthUser, roll: &RoomRoll) -> RoomRollSummary {
    RoomRollSummary {
        id: roll.id,
        user_id: roll.user_id,
        username: user.username.clone(),
        roll_expression: roll.roll_expression.clone(),
        roll_result: roll.roll_result.clone(),
        final_result: roll.final_result,
        created_at: roll.created_at,
        updated_at: roll.updated_at,
    }
}

fn sse_event(event: &RoomStreamEvent) -> Event {
    let payload = serde_json::to_string(event)
        .unwrap_or_else(|error| format!(r#"{{"type":"access_revoked","reason":"{error}"}}"#));

    Event::default()
        .event(room_stream_event_name(event))
        .data(payload)
}

fn room_stream_event_name(event: &RoomStreamEvent) -> &'static str {
    match event {
        RoomStreamEvent::Snapshot { .. } => "snapshot",
        RoomStreamEvent::RosterChanged { .. } => "roster_changed",
        RoomStreamEvent::RollCreated { .. } => "roll_created",
        RoomStreamEvent::AccessRevoked { .. } => "access_revoked",
    }
}
