#[cfg(feature = "hydrate")]
use std::{cell::RefCell, rc::Rc};

use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_params_map;

#[cfg(feature = "hydrate")]
use crate::{
    client::utils::rooms::{
        IntervalHandle, RoomEventStream, append_live_room_roll, room_roll_feed_from_page,
    },
    shared::data::room::RoomStreamEvent,
};
use crate::{
    client::{
        components::{
            active_user_feed::ActiveUserFeed,
            add_room_member::AddRoomMember,
            bottom_roll_composer::BottomRollComposer,
            dialog::Dialog,
            roll_editor::{RollEditor, RollEditorController},
            roll_feed::RollFeed,
        },
        context::page_title::{NOT_FOUND_PAGE_TITLE, ROOMS_PAGE_TITLE, use_page_title_context},
        utils::{
            async_state::{LoadState, MutationState},
            roll_feed::DiceRollFeed,
            rooms::{
                add_room_roll_request, allow_member_request, get_room_access_request,
                kick_member_request, list_room_rolls_request, prepend_room_roll_page,
            },
        },
    },
    shared::data::{
        room::{
            Room, RoomId, RoomMemberSummary, RoomRollId, RoomRosterMember, RoomViewerState,
            RoomViewerStatus,
        },
        user::UserId,
    },
};

stylance::import_style!(style, "room.module.scss");

type RoomAccessState = LoadState<RoomViewerState, String>;

#[cfg(feature = "hydrate")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoomSubscriptionTarget {
    Stream(RoomId),
    Pending(RoomId),
    None,
}

fn page_title_for_access_state(access_state: RoomAccessState) -> String {
    match access_state {
        LoadState::Idle | LoadState::Loading => ROOMS_PAGE_TITLE.to_string(),
        LoadState::Ready(viewer_state) => viewer_state.room.name,
        LoadState::Error(_) => NOT_FOUND_PAGE_TITLE.to_string(),
    }
}

fn membership_badge_copy(can_manage_members: bool) -> &'static str {
    if can_manage_members {
        "Admin"
    } else {
        "Joined"
    }
}

fn stream_badge_copy(connected: bool) -> &'static str {
    if connected {
        "Live stream open"
    } else {
        "Live stream reconnecting"
    }
}

#[derive(Clone)]
struct RoomAccessController {
    access_state: RwSignal<RoomAccessState>,
    current_room_id: RwSignal<Option<RoomId>>,
}

impl RoomAccessController {
    fn new() -> Self {
        Self {
            access_state: RwSignal::new(LoadState::idle()),
            current_room_id: RwSignal::new(None),
        }
    }
}

#[derive(Clone)]
struct RoomLiveState {
    room: RwSignal<Option<Room>>,
    roster_members: RwSignal<Vec<RoomRosterMember>>,
    roll_feed: RwSignal<DiceRollFeed>,
    next_before_id: RwSignal<Option<RoomRollId>>,
    loading_more: RwSignal<bool>,
    stream_connected: RwSignal<bool>,
    live_ready: RwSignal<bool>,
    stream_error: RwSignal<Option<String>>,
}

impl RoomLiveState {
    fn new() -> Self {
        Self {
            room: RwSignal::new(None),
            roster_members: RwSignal::new(Vec::new()),
            roll_feed: RwSignal::new(DiceRollFeed::new()),
            next_before_id: RwSignal::new(None),
            loading_more: RwSignal::new(false),
            stream_connected: RwSignal::new(false),
            live_ready: RwSignal::new(false),
            stream_error: RwSignal::new(None),
        }
    }

    fn reset(&self) {
        self.room.set(None);
        self.roster_members.set(Vec::new());
        self.roll_feed.set(DiceRollFeed::new());
        self.next_before_id.set(None);
        self.loading_more.set(false);
        self.stream_connected.set(false);
        self.live_ready.set(false);
        self.stream_error.set(None);
    }
}

#[derive(Clone)]
struct RoomMemberState {
    action_state: RwSignal<MutationState<String>>,
    action_busy_user_id: RwSignal<Option<i64>>,
    kick_dialog_member: RwSignal<Option<RoomMemberSummary>>,
}

impl RoomMemberState {
    fn new() -> Self {
        Self {
            action_state: RwSignal::new(MutationState::idle()),
            action_busy_user_id: RwSignal::new(None),
            kick_dialog_member: RwSignal::new(None),
        }
    }

    fn reset(&self) {
        self.action_state.set(MutationState::idle());
        self.action_busy_user_id.set(None);
        self.kick_dialog_member.set(None);
    }

    fn request_kick(&self, roster_members: &[RoomRosterMember], user_id: UserId) {
        let selected_member = roster_members
            .iter()
            .find(|member| !member.is_creator && member.user_id == user_id)
            .map(|member| RoomMemberSummary {
                user_id: member.user_id,
                username: member.username.clone(),
                status: member.status,
            });

        self.action_state.set(MutationState::idle());
        self.kick_dialog_member.set(selected_member);
    }

    fn cancel_kick(&self) {
        self.kick_dialog_member.set(None);
    }

    fn action_error_signal(&self) -> Signal<Option<String>> {
        let members = self.clone();
        Signal::derive(move || members.action_state.get().as_error().cloned())
    }
}

#[derive(Clone)]
struct RoomRollState {
    editor: RollEditorController,
    submit_state: RwSignal<MutationState<String>>,
}

impl RoomRollState {
    fn new() -> Self {
        Self {
            editor: RollEditorController::new(),
            submit_state: RwSignal::new(MutationState::idle()),
        }
    }

    fn reset(&self) {
        self.submit_state.set(MutationState::idle());
    }

    fn error_signal(&self) -> Signal<Option<String>> {
        let rolls = self.clone();
        Signal::derive(move || rolls.submit_state.get().as_error().cloned())
    }
}

#[derive(Clone)]
struct RoomPageState {
    access: RoomAccessController,
    live: RoomLiveState,
    members: RoomMemberState,
    rolls: RoomRollState,
}

impl RoomPageState {
    fn new() -> Self {
        Self {
            access: RoomAccessController::new(),
            live: RoomLiveState::new(),
            members: RoomMemberState::new(),
            rolls: RoomRollState::new(),
        }
    }

    fn reset_for_room_change(&self) {
        self.access.current_room_id.set(None);
        self.access.access_state.set(LoadState::idle());
        self.live.reset();
        self.members.reset();
        self.rolls.reset();
    }
}

#[derive(Clone)]
struct RoomPageActions {
    load_older_rolls: Callback<()>,
    on_roll: Callback<String>,
    on_allow_member: Callback<UserId>,
    on_request_kick: Callback<UserId>,
    on_cancel_kick: Callback<()>,
    on_confirm_kick: Callback<()>,
}

#[component]
fn FeedbackMessage(message: String) -> impl IntoView {
    view! { <p class=style::page_feedback>{message}</p> }
}

#[component]
fn RoomStateCard(label: &'static str, title: String, summary: String) -> impl IntoView {
    view! {
        <section class=format!("g-panel g-panel-strong {}", style::state_card)>
            <p class="g-section-label">{label}</p>
            <h1 class=style::room_title>{title}</h1>
            <p class=style::room_summary>{summary}</p>
        </section>
    }
}

#[component]
fn PendingRoomState(
    viewer: RoomViewerState,
    #[prop(into)] stream_error: Signal<Option<String>>,
) -> impl IntoView {
    let room_id = viewer.room.id.into_inner();

    view! {
        <section class=format!("g-panel g-panel-strong {}", style::state_card)>
            <p class="g-section-label">"Waiting for approval"</p>
            <h1 class=style::room_title>{viewer.room.name.clone()}</h1>
            <p class=style::room_summary>
                "Your join request is in. Stay on this page while the room admin approves access."
            </p>
            <div class=style::room_header_meta>
                <span class=style::room_id_badge>{format!("#{room_id}")}</span>
                <span class=style::room_note_badge>"Pending"</span>
            </div>
            {move || { stream_error.get().map(|message| view! { <FeedbackMessage message /> }) }}
        </section>
    }
}

#[component]
fn KickedRoomState(viewer: RoomViewerState) -> impl IntoView {
    let room_id = viewer.room.id.into_inner();

    view! {
        <section class=format!("g-panel g-panel-strong {}", style::state_card)>
            <p class="g-section-label">"Room access revoked"</p>
            <h1 class=style::room_title>{viewer.room.name.clone()}</h1>
            <p class=style::room_summary>
                "You no longer have access to this room. Head back to the rooms board for another table."
            </p>
            <div class=style::room_header_meta>
                <span class=style::room_id_badge>{format!("#{room_id}")}</span>
                <span class=style::room_note_badge>"Kicked"</span>
            </div>
        </section>
    }
}

#[component]
fn RoomEditorPane(
    live: RoomLiveState,
    rolls: RoomRollState,
    #[prop(into)] on_roll: Callback<String>,
) -> impl IntoView {
    let roll_error = rolls.error_signal();

    view! {
        <section class=format!(
            "{} {}",
            style::room_main,
            style::hide_on_mobile,
        )>
            {move || {
                if !live.live_ready.get() {
                    view! {
                        <section class=format!("g-panel g-panel-strong {}", style::state_card)>
                            <p class="g-section-label">"Opening stream"</p>
                            <h2 class=style::state_title>"Loading room state."</h2>
                        </section>
                    }
                        .into_any()
                } else {
                    view! {
                        <>
                            {move || {
                                live.stream_error
                                    .get()
                                    .map(|message| view! { <FeedbackMessage message /> })
                            }}
                            {move || {
                                roll_error
                                    .get()
                                    .map(|message| view! { <FeedbackMessage message /> })
                            }} <div class=style::room_inline_editor>
                                <RollEditor
                                    controller=rolls.editor
                                    on_roll
                                    expression_input_id="room-editor-expression-input".to_string()
                                />
                            </div>
                        </>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

#[component]
fn RoomSidebar(
    viewer: RoomViewerState,
    room: Room,
    live: RoomLiveState,
    members: RoomMemberState,
    actions: RoomPageActions,
) -> impl IntoView {
    let action_error = members.action_error_signal();

    view! {
        <aside class=style::room_rail>
            <section class=format!("g-panel g-panel-strong {}", style::room_header)>
                <div class=style::room_header_top>
                    <p class="g-section-label">"Table ledger"</p>
                    <h2 class=style::room_title>{room.name.clone()}</h2>
                    <p class=style::room_summary>"Shared room with a live roll feed."</p>
                </div>

                <div class=style::room_header_meta>
                    <span class=style::room_note_badge>
                        {membership_badge_copy(viewer.can_manage_members)}
                    </span>
                    <span
                        class=style::stream_badge
                        data-connected=move || {
                            if live.stream_connected.get() { "true" } else { "false" }
                        }
                    >
                        {move || stream_badge_copy(live.stream_connected.get())}
                    </span>
                </div>
            </section>

            <RollFeed
                feed=live.roll_feed
                loading_more=live.loading_more
                load_older_rolls=actions.load_older_rolls
            />

            <Show when=move || viewer.can_manage_members>
                <AddRoomMember room_id=room.id />
            </Show>

            <ActiveUserFeed
                roster_members=live.roster_members
                connected=live.stream_connected
                can_manage_members=viewer.can_manage_members
                busy_user_id=members.action_busy_user_id
                action_error=action_error
                on_allow=actions.on_allow_member
                on_request_kick=actions.on_request_kick
            />
        </aside>
    }
}

#[component]
fn ActiveRoomLayout(
    viewer: RoomViewerState,
    room: Room,
    state: RoomPageState,
    actions: RoomPageActions,
) -> impl IntoView {
    view! {
        <div class=format!("g-page-shell-split {}", style::room_layout)>
            <RoomEditorPane
                live=state.live.clone()
                rolls=state.rolls.clone()
                on_roll=actions.on_roll.clone()
            />
            <RoomSidebar
                viewer
                room
                live=state.live.clone()
                members=state.members.clone()
                actions=actions.clone()
            />
        </div>
    }
}

#[component]
fn KickMemberDialog(members: RoomMemberState, actions: RoomPageActions) -> impl IntoView {
    view! {
        <Dialog
            open=move || members.kick_dialog_member.get().is_some()
            label="Member controls"
            title="Confirm kick".to_string()
            summary="Kicked users lose access immediately and stay visible in the kicked list so they can be reinstated later."
                .to_string()
            on_close=actions.on_cancel_kick
        >
            <p class=style::room_summary>
                {move || {
                    members
                        .kick_dialog_member
                        .get()
                        .map(|member| {
                            format!(
                                "{} will lose access to the room immediately.",
                                member.username.as_str(),
                            )
                        })
                        .unwrap_or_default()
                }}
            </p>
            <div class=style::dialog_actions>
                <button
                    class="g-button-ghost"
                    type="button"
                    on:click=move |_| actions.on_cancel_kick.run(())
                >
                    "Cancel"
                </button>
                <button
                    class="g-button-action"
                    type="button"
                    on:click=move |_| actions.on_confirm_kick.run(())
                >
                    "Confirm kick"
                </button>
            </div>
        </Dialog>
    }
}

fn room_page_content(state: RoomPageState, actions: RoomPageActions) -> impl IntoView {
    let access_state = state.access.clone();
    let pending_live = state.live.clone();
    let active_room_state = state.live.clone();
    let layout_state = state.clone();
    let layout_actions = actions.clone();
    let kick_members = state.members.clone();
    let kick_actions = actions.clone();
    let composer_state = state.clone();
    let composer_actions = actions.clone();

    view! {
        <>
            <section class="g-page g-page-shell">
                <section class=format!("g-panel g-panel-strong {}", style::room_shell)>
                    <div class=style::hide_on_mobile>
                        <div class="g-page-meta">
                            <a class="g-button-utility" href="/rooms">
                                "Back to rooms"
                            </a>
                        </div>
                    </div>

                    {move || match access_state.access_state.get() {
                        LoadState::Idle | LoadState::Loading => {
                            view! {
                                <RoomStateCard
                                    label="Room access"
                                    title="Loading room...".to_string()
                                    summary="Checking your access and opening the live room stream."
                                        .to_string()
                                />
                            }
                                .into_any()
                        }
                        LoadState::Error(message) => {
                            view! {
                                <RoomStateCard
                                    label="Room lookup"
                                    title="Room unavailable.".to_string()
                                    summary=message
                                />
                            }
                                .into_any()
                        }
                        LoadState::Ready(viewer) => {
                            match viewer.viewer_status {
                                RoomViewerStatus::Pending => {
                                    view! {
                                        <PendingRoomState
                                            viewer
                                            stream_error=pending_live.stream_error
                                        />
                                    }
                                        .into_any()
                                }
                                RoomViewerStatus::Kicked => {
                                    view! { <KickedRoomState viewer /> }.into_any()
                                }
                                RoomViewerStatus::Creator | RoomViewerStatus::Joined => {
                                    let active_room = active_room_state
                                        .room
                                        .get()
                                        .unwrap_or(viewer.room.clone());

                                    view! {
                                        <ActiveRoomLayout
                                            viewer
                                            room=active_room
                                            state=layout_state.clone()
                                            actions=layout_actions.clone()
                                        />
                                    }
                                        .into_any()
                                }
                            }
                        }
                    }}
                </section>

                <KickMemberDialog members=kick_members actions=kick_actions />
            </section>
            <Show when=move || {
                matches!(
                    composer_state.access.access_state.get(),
                    LoadState::Ready(
                        RoomViewerState {
                            viewer_status: RoomViewerStatus::Creator | RoomViewerStatus::Joined,
                            ..
                        },
                    )
                ) && composer_state.live.live_ready.get()
            }>
                <BottomRollComposer
                    controller=composer_state.rolls.editor.clone()
                    expression_input_id="room-mobile-expression-input".to_string()
                    on_roll=composer_actions.on_roll.clone()
                    error=move || composer_state.rolls.submit_state.get().as_error().cloned()
                    dialog_title="Edit room roll".to_string()
                    dialog_summary="Update the current room expression or load a preset, then confirm to return to the feed."
                        .to_string()
                />
            </Show>
        </>
    }
}

#[component]
pub fn RoomPage() -> impl IntoView {
    let page_title = use_page_title_context();
    let params = use_params_map();
    let state = RoomPageState::new();

    #[cfg(feature = "hydrate")]
    let room_stream = Rc::new(RefCell::new(None::<RoomEventStream>));
    #[cfg(feature = "hydrate")]
    let pending_poll = Rc::new(RefCell::new(None::<IntervalHandle>));
    #[cfg(feature = "hydrate")]
    let subscription_target = RwSignal::new(RoomSubscriptionTarget::None);

    {
        let state = state.clone();
        Effect::new(move |_| {
            page_title.set(page_title_for_access_state(state.access.access_state.get()));
        });
    }

    {
        let state = state.clone();
        let params = params.clone();
        Effect::new(move |_| {
            let raw_room_id = params.get().get("roomId").unwrap_or_default();

            state.reset_for_room_change();

            let trimmed_room_id = raw_room_id.trim().to_string();
            if trimmed_room_id.is_empty() {
                state
                    .access
                    .access_state
                    .set(LoadState::error("Room not found.".to_string()));
                return;
            }

            let Ok(room_id) = trimmed_room_id.parse::<i64>() else {
                state
                    .access
                    .access_state
                    .set(LoadState::error("Room IDs use digits only.".to_string()));
                return;
            };

            let room_id = RoomId(room_id);
            state.access.current_room_id.set(Some(room_id));
            state.access.access_state.set(LoadState::loading());

            let state = state.clone();
            spawn_local(async move {
                match get_room_access_request(room_id.into_inner()).await {
                    Ok(viewer_state) => {
                        state.live.room.set(Some(viewer_state.room.clone()));
                        state
                            .access
                            .access_state
                            .set(LoadState::ready(viewer_state));
                    }
                    Err(message) => state.access.access_state.set(LoadState::error(message)),
                }
            });
        });
    }

    #[cfg(feature = "hydrate")]
    Effect::new({
        let room_stream = room_stream.clone();
        let pending_poll = pending_poll.clone();
        let state = state.clone();

        move |_| {
            let next_target = match (
                state.access.current_room_id.get(),
                state.access.access_state.get(),
            ) {
                (Some(room_id), LoadState::Ready(viewer_state)) => {
                    match viewer_state.viewer_status {
                        RoomViewerStatus::Creator | RoomViewerStatus::Joined => {
                            RoomSubscriptionTarget::Stream(room_id)
                        }
                        RoomViewerStatus::Pending => RoomSubscriptionTarget::Pending(room_id),
                        RoomViewerStatus::Kicked => RoomSubscriptionTarget::None,
                    }
                }
                _ => RoomSubscriptionTarget::None,
            };

            if subscription_target.get_untracked() == next_target {
                return;
            }

            room_stream.borrow_mut().take();
            pending_poll.borrow_mut().take();
            subscription_target.set(next_target);

            match next_target {
                RoomSubscriptionTarget::Stream(room_id) => {
                    state.live.live_ready.set(false);
                    state.live.stream_error.set(None);

                    let stream = RoomEventStream::connect(
                        room_id,
                        {
                            move |event| match event {
                                RoomStreamEvent::Snapshot { snapshot } => {
                                    state.live.room.set(Some(snapshot.room.clone()));
                                    state.live.roster_members.set(snapshot.roster_members);
                                    state
                                        .live
                                        .next_before_id
                                        .set(snapshot.recent_rolls.next_before_id);
                                    state
                                        .live
                                        .roll_feed
                                        .set(room_roll_feed_from_page(&snapshot.recent_rolls));
                                    state.live.live_ready.set(true);
                                    state.live.stream_connected.set(true);

                                    state.access.access_state.update(|access_state| {
                                        if let LoadState::Ready(viewer_state) = access_state {
                                            viewer_state.room = snapshot.room.clone();
                                            viewer_state.can_manage_members =
                                                snapshot.can_manage_members;
                                        }
                                    });
                                }
                                RoomStreamEvent::RosterChanged {
                                    roster_members: next_members,
                                } => {
                                    state.live.roster_members.set(next_members);
                                    state.live.stream_connected.set(true);
                                }
                                RoomStreamEvent::RollCreated { roll } => {
                                    state
                                        .live
                                        .roll_feed
                                        .update(|feed| append_live_room_roll(feed, &roll));
                                    state.live.stream_connected.set(true);
                                }
                                RoomStreamEvent::AccessRevoked { reason } => {
                                    state.live.stream_connected.set(false);
                                    state.live.stream_error.set(Some(reason.clone()));
                                    state.live.live_ready.set(false);
                                    state.live.roster_members.set(Vec::new());
                                    state.live.roll_feed.set(DiceRollFeed::new());
                                    state.live.next_before_id.set(None);

                                    if let Some(room) = state.live.room.get_untracked() {
                                        state.access.access_state.set(LoadState::ready(
                                            RoomViewerState {
                                                room,
                                                viewer_status: RoomViewerStatus::Kicked,
                                                can_manage_members: false,
                                            },
                                        ));
                                    } else {
                                        state.access.access_state.set(LoadState::error(reason));
                                    }
                                }
                            }
                        },
                        move |connected| {
                            state.live.stream_connected.set(connected);
                            if connected {
                                state.live.stream_error.set(None);
                            }
                        },
                        move |message| {
                            state.live.stream_connected.set(false);
                            state.live.stream_error.set(Some(message));
                        },
                    );

                    match stream {
                        Ok(stream) => {
                            room_stream.borrow_mut().replace(stream);
                        }
                        Err(message) => {
                            state.live.stream_connected.set(false);
                            state.live.stream_error.set(Some(message));
                        }
                    }
                }
                RoomSubscriptionTarget::Pending(room_id) => {
                    state.live.stream_connected.set(false);
                    state.live.live_ready.set(false);

                    let poll = IntervalHandle::start(4_000, {
                        move || {
                            spawn_local(async move {
                                match get_room_access_request(room_id.into_inner()).await {
                                    Ok(viewer_state) => {
                                        state.live.room.set(Some(viewer_state.room.clone()));
                                        state
                                            .access
                                            .access_state
                                            .set(LoadState::ready(viewer_state));
                                        state.live.stream_error.set(None);
                                    }
                                    Err(message) => state.live.stream_error.set(Some(message)),
                                }
                            });
                        }
                    });

                    match poll {
                        Ok(poll) => {
                            pending_poll.borrow_mut().replace(poll);
                        }
                        Err(message) => state.live.stream_error.set(Some(message)),
                    }
                }
                RoomSubscriptionTarget::None => {
                    state.live.stream_connected.set(false);
                    state.live.live_ready.set(false);
                }
            }
        }
    });

    let on_roll = {
        let state = state.clone();
        Callback::new(move |expression: String| {
            let Some(room_id) = state.access.current_room_id.get_untracked() else {
                return;
            };

            state.rolls.submit_state.set(MutationState::pending());
            let state = state.clone();
            spawn_local(async move {
                if let Err(message) = add_room_roll_request(room_id.into_inner(), &expression).await
                {
                    state.rolls.submit_state.set(MutationState::error(message));
                } else {
                    state.rolls.submit_state.set(MutationState::success());
                }
            });
        })
    };

    let load_older_rolls = {
        let state = state.clone();
        Callback::new(move |_| {
            let Some(room_id) = state.access.current_room_id.get_untracked() else {
                return;
            };
            let Some(before_id) = state.live.next_before_id.get_untracked() else {
                return;
            };
            if state.live.loading_more.get_untracked() {
                return;
            }

            state.live.loading_more.set(true);
            state.live.stream_error.set(None);

            let state = state.clone();
            spawn_local(async move {
                match list_room_rolls_request(room_id.into_inner(), Some(before_id)).await {
                    Ok(page) => {
                        state.live.next_before_id.set(page.next_before_id);
                        state
                            .live
                            .roll_feed
                            .update(|feed| prepend_room_roll_page(feed, &page));
                    }
                    Err(message) => state.live.stream_error.set(Some(message)),
                }
                state.live.loading_more.set(false);
            });
        })
    };

    let on_allow_member = {
        let state = state.clone();
        Callback::new(move |user_id: UserId| {
            let Some(room_id) = state.access.current_room_id.get_untracked() else {
                return;
            };
            let user_id_value = user_id.into_inner();

            state.members.action_state.set(MutationState::pending());
            state.members.action_busy_user_id.set(Some(user_id_value));

            let members = state.members.clone();
            spawn_local(async move {
                if let Err(message) =
                    allow_member_request(room_id.into_inner(), user_id_value).await
                {
                    members.action_state.set(MutationState::error(message));
                } else {
                    members.action_state.set(MutationState::success());
                }
                members.action_busy_user_id.set(None);
            });
        })
    };

    let on_request_kick = {
        let state = state.clone();
        Callback::new(move |user_id: UserId| {
            state
                .members
                .request_kick(&state.live.roster_members.get_untracked(), user_id);
        })
    };

    let on_cancel_kick = {
        let state = state.clone();
        Callback::new(move |_| {
            state.members.cancel_kick();
        })
    };

    let on_confirm_kick = {
        let state = state.clone();
        Callback::new(move |_| {
            let Some(room_id) = state.access.current_room_id.get_untracked() else {
                return;
            };
            let Some(member) = state.members.kick_dialog_member.get_untracked() else {
                return;
            };

            let user_id = member.user_id.into_inner();
            state.members.action_state.set(MutationState::pending());
            state.members.action_busy_user_id.set(Some(user_id));

            let members = state.members.clone();
            spawn_local(async move {
                if let Err(message) = kick_member_request(room_id.into_inner(), user_id).await {
                    members.action_state.set(MutationState::error(message));
                } else {
                    members.action_state.set(MutationState::success());
                    members.cancel_kick();
                }
                members.action_busy_user_id.set(None);
            });
        })
    };

    let actions = RoomPageActions {
        load_older_rolls,
        on_roll,
        on_allow_member,
        on_request_kick,
        on_cancel_kick,
        on_confirm_kick,
    };

    room_page_content(state, actions)
}
