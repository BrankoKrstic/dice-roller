use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use crate::{
    client::{
        components::dialog::Dialog,
        context::page_title::use_static_page_title,
        utils::rooms::{
            MAX_ACTIVE_CREATED_ROOMS, archive_room_request, join_room_request,
            latest_roll_activity_line, leave_room_request, list_joined_rooms_request,
            parse_room_id_input, room_route,
        },
    },
    shared::data::room::JoinedRoomSummary,
};

stylance::import_style!(style, "rooms.module.scss");

#[derive(Debug, Clone, PartialEq)]
enum JoinedRoomsState {
    Loading,
    Loaded(Vec<JoinedRoomSummary>),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
enum RoomsActionDialog {
    Leave(JoinedRoomSummary),
    Archive(JoinedRoomSummary),
}

#[derive(Clone)]
struct RoomsBoardState {
    join_input: RwSignal<String>,
    join_error: RwSignal<Option<String>>,
    joining: RwSignal<bool>,
    rooms_state: RwSignal<JoinedRoomsState>,
    action_error: RwSignal<Option<String>>,
    action_busy_room_id: RwSignal<Option<i64>>,
    action_dialog: RwSignal<Option<RoomsActionDialog>>,
}

impl RoomsBoardState {
    fn new() -> Self {
        Self {
            join_input: RwSignal::new(String::new()),
            join_error: RwSignal::new(None),
            joining: RwSignal::new(false),
            rooms_state: RwSignal::new(JoinedRoomsState::Loading),
            action_error: RwSignal::new(None),
            action_busy_room_id: RwSignal::new(None),
            action_dialog: RwSignal::new(None),
        }
    }

    fn request_leave(&self, room: JoinedRoomSummary) {
        self.action_error.set(None);
        self.action_dialog.set(Some(RoomsActionDialog::Leave(room)));
    }

    fn request_archive(&self, room: JoinedRoomSummary) {
        self.action_error.set(None);
        self.action_dialog
            .set(Some(RoomsActionDialog::Archive(room)));
    }

    fn cancel_action(&self) {
        self.action_dialog.set(None);
    }
}

#[derive(Clone)]
struct RoomsBoardActions {
    request_leave: Callback<JoinedRoomSummary>,
    request_archive: Callback<JoinedRoomSummary>,
    cancel_action: Callback<()>,
    confirm_action: Callback<()>,
}

fn active_created_room_count(rooms: &[JoinedRoomSummary]) -> usize {
    rooms.iter().filter(|room| room.can_manage_members).count()
}

fn create_room_limit_reached(rooms_state: &JoinedRoomsState) -> bool {
    matches!(
        rooms_state,
        JoinedRoomsState::Loaded(rooms)
            if active_created_room_count(rooms) >= MAX_ACTIVE_CREATED_ROOMS
    )
}

fn create_room_helper_copy(rooms_state: &JoinedRoomsState) -> String {
    if create_room_limit_reached(rooms_state) {
        format!(
            "You can create up to {MAX_ACTIVE_CREATED_ROOMS} active rooms. Archive one first to make space.",
        )
    } else {
        "You can keep up to five active rooms at a time.".to_string()
    }
}

fn room_role_copy(room: &JoinedRoomSummary) -> &'static str {
    if room.can_manage_members {
        "Admin"
    } else {
        "Joined table"
    }
}

fn room_member_copy(room: &JoinedRoomSummary) -> String {
    if room.active_member_count == 0 {
        "Nobody is connected right now.".to_string()
    } else {
        format!(
            "{} player{} connected to the room stream.",
            room.active_member_count,
            if room.active_member_count == 1 {
                ""
            } else {
                "s"
            },
        )
    }
}

impl RoomsActionDialog {
    fn room(&self) -> &JoinedRoomSummary {
        match self {
            Self::Leave(room) | Self::Archive(room) => room,
        }
    }

    fn summary_copy(&self) -> String {
        match self {
            Self::Leave(room) => {
                format!(
                    "Leave {} and remove it from your active rooms?",
                    room.room.name
                )
            }
            Self::Archive(room) => {
                format!(
                    "Archive {} and make it unavailable to everyone?",
                    room.room.name,
                )
            }
        }
    }

    fn confirm_label(&self) -> &'static str {
        match self {
            Self::Leave(_) => "Confirm leave",
            Self::Archive(_) => "Delete room",
        }
    }
}

#[component]
fn EmptyState(title: &'static str, copy: String) -> impl IntoView {
    view! {
        <div class=style::empty_state>
            <h3 class=style::empty_state_title>{title}</h3>
            <p class=style::empty_state_copy>{copy}</p>
        </div>
    }
}

#[component]
fn CreateRoomCard(#[prop(into)] rooms_state: Signal<JoinedRoomsState>) -> impl IntoView {
    let create_disabled = Signal::derive(move || create_room_limit_reached(&rooms_state.get()));
    let helper_copy = Signal::derive(move || create_room_helper_copy(&rooms_state.get()));

    view! {
        <article class=style::launch_card>
            <p class="g-section-label">"Start a room"</p>
            <h2 class=style::launch_card_title>"Create a room"</h2>
            <p class=style::launch_card_summary>"Spin up a new shared roll feed."</p>
            <div class=style::launch_action_row>
                {move || {
                    if create_disabled.get() {
                        view! {
                            <button
                                id="rooms-create-room"
                                class="g-button-action"
                                type="button"
                                disabled=true
                            >
                                "Create a room"
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <a id="rooms-create-room" class="g-button-action" href="/rooms/create">
                                "Create a room"
                            </a>
                        }
                            .into_any()
                    }
                }}
            </div>
            <p class=style::launch_helper>{move || helper_copy.get()}</p>
        </article>
    }
}

#[component]
fn JoinRoomCard(board: RoomsBoardState) -> impl IntoView {
    view! {
        <article class=style::launch_card>
            <p class="g-section-label">"Join a table"</p>
            <h2 class=style::launch_card_title>"Join by room ID"</h2>
            <label class="g-field-label" for="rooms-join-room-id">
                "Room ID"
            </label>
            <div class=style::join_row>
                <input
                    id="rooms-join-room-id"
                    class="g-text-input"
                    type="text"
                    inputmode="numeric"
                    placeholder="42"
                    prop:value=move || board.join_input.get()
                    on:input=move |event| {
                        board.join_error.set(None);
                        board.join_input.set(event_target_value(&event));
                    }
                />
                <div class=style::join_action_slot>
                    <button
                        id="rooms-join-submit"
                        class="g-button-action"
                        type="submit"
                        disabled=move || board.joining.get()
                    >
                        {move || if board.joining.get() { "Joining..." } else { "Join room" }}
                    </button>
                </div>
            </div>
            <p class=style::join_helper>"Pending users wait there until admitted."</p>
            {move || {
                board
                    .join_error
                    .get()
                    .map(|message| view! { <p class=style::join_feedback>{message}</p> })
            }}
        </article>
    }
}

#[component]
fn JoinedRoomCard(
    room: JoinedRoomSummary,
    board: RoomsBoardState,
    actions: RoomsBoardActions,
) -> impl IntoView {
    let room_id = room.room.id.into_inner();
    let room_target = room_route(room_id);
    let activity_line = latest_roll_activity_line(&room.latest_roll);
    let archive_board = board.clone();
    let leave_board = board.clone();
    let archive_actions = actions.clone();
    let leave_actions = actions.clone();

    view! {
        <article class=style::room_card>
            <div class=style::room_card_header>
                <div class=style::room_identity>
                    <p class="g-section-label">{room_role_copy(&room)}</p>
                    <h3 class=style::room_title>{room.room.name.clone()}</h3>
                </div>
                <span class=style::room_id_badge>{format!("#{room_id}")}</span>
            </div>

            <dl class=style::room_meta_list>
                <div class=style::room_meta_row>
                    <dt>"Who is here"</dt>
                    <dd>{room_member_copy(&room)}</dd>
                </div>
                <div class=style::room_meta_row>
                    <dt>"Latest activity"</dt>
                    <dd>{activity_line}</dd>
                </div>
            </dl>

            <div class=style::room_card_footer>
                <a class="g-button-action" href=room_target>
                    "Enter room"
                </a>
                {if room.can_manage_members {
                    view! {
                        <button
                            class="g-button-ghost"
                            type="button"
                            prop:disabled=move || archive_board.action_busy_room_id.get() == Some(room_id)
                            on:click={
                                let request_archive = archive_actions.request_archive.clone();
                                let room = room.clone();
                                move |_| request_archive.run(room.clone())
                            }
                        >
                            "Delete room"
                        </button>
                    }
                        .into_any()
                } else {
                    view! {
                        <button
                            class="g-button-ghost"
                            type="button"
                            prop:disabled=move || leave_board.action_busy_room_id.get() == Some(room_id)
                            on:click={
                                let request_leave = leave_actions.request_leave.clone();
                                let room = room.clone();
                                move |_| request_leave.run(room.clone())
                            }
                        >
                            "Leave room"
                        </button>
                    }
                        .into_any()
                }}
            </div>
        </article>
    }
}

fn rooms_page_content(board: RoomsBoardState, actions: RoomsBoardActions) -> impl IntoView {
    let rooms_board = board.clone();
    let card_board = board.clone();
    let card_actions = actions.clone();
    let dialog_board = board.clone();
    let dialog_actions = actions.clone();
    let cancel_actions = actions.clone();
    let confirm_actions = actions.clone();

    view! {
        <section class="g-page g-page-shell">
            <section class=format!("g-panel g-panel-strong {}", style::launch_panel)>
                <div class=style::launch_header>
                    <p class="g-section-label">"Rooms"</p>
                    <h1 class="g-section-title">"Choose a room to roll int."</h1>
                    <p class="g-section-summary">
                        "Create a new room, or join an existing one to roll dice in a group."
                    </p>
                </div>

                <div class=style::launch_grid>
                    <CreateRoomCard rooms_state=board.rooms_state />
                    <JoinRoomCard board=board.clone() />
                </div>
            </section>

            <section class=format!("g-panel g-panel-strong {}", style::joined_rooms_panel)>
                <div class=style::joined_rooms_header>
                    <p class="g-section-label">"Your rooms"</p>
                    <h2 class="g-section-title">"Joined rooms"</h2>
                    <p class="g-section-summary">"Rooms you've joined."</p>
                </div>

                {move || {
                    card_board
                        .action_error
                        .get()
                        .map(|message| view! { <p class=style::join_feedback>{message}</p> })
                }}

                {move || match rooms_board.rooms_state.get() {
                    JoinedRoomsState::Loading => {
                        view! {
                            <EmptyState
                                title="Loading joined rooms..."
                                copy="Pulling your current room list from the server.".to_string()
                            />
                        }
                            .into_any()
                    }
                    JoinedRoomsState::Error(message) => {
                        view! { <EmptyState title="Could not load joined rooms." copy=message /> }
                            .into_any()
                    }
                    JoinedRoomsState::Loaded(rooms) => {
                        if rooms.is_empty() {
                            view! {
                                <EmptyState
                                    title="No joined rooms yet."
                                    copy="Create a room or request to join one by ID to build your active board."
                                        .to_string()
                                />
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class=style::rooms_grid>
                                    {rooms
                                        .into_iter()
                                        .map(|room| {
                                            view! {
                                                <JoinedRoomCard
                                                    room
                                                    board=card_board.clone()
                                                    actions=card_actions.clone()
                                                />
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </section>

            <Dialog
                open=move || dialog_board.action_dialog.get().is_some()
                label="Room actions"
                title="Confirm room action".to_string()
                on_close=dialog_actions.cancel_action.clone()
            >
                <p class=style::join_helper>
                    {move || {
                        dialog_board
                            .action_dialog
                            .get()
                            .map(|dialog| dialog.summary_copy())
                            .unwrap_or_default()
                    }}
                </p>
                <div class=style::room_card_footer>
                    <button
                        class="g-button-ghost"
                        type="button"
                        on:click=move |_| cancel_actions.cancel_action.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="g-button-action"
                        type="button"
                        on:click=move |_| confirm_actions.confirm_action.run(())
                    >
                        {move || {
                            dialog_board
                                .action_dialog
                                .get()
                                .map(|dialog| dialog.confirm_label())
                                .unwrap_or("")
                        }}
                    </button>
                </div>
            </Dialog>
        </section>
    }
}

#[component]
pub fn RoomsPage() -> impl IntoView {
    use_static_page_title("Rooms");

    let navigate = use_navigate();
    let board = RoomsBoardState::new();

    let refresh_rooms = {
        let board = board.clone();
        Callback::new(move |_| {
            board.rooms_state.set(JoinedRoomsState::Loading);
            spawn_local(async move {
                match list_joined_rooms_request().await {
                    Ok(rooms) => board.rooms_state.set(JoinedRoomsState::Loaded(rooms)),
                    Err(message) => board.rooms_state.set(JoinedRoomsState::Error(message)),
                }
            });
        })
    };

    {
        let refresh_rooms = refresh_rooms.clone();
        Effect::new(move |_| {
            if !cfg!(feature = "hydrate") {
                return;
            }

            refresh_rooms.run(());
        });
    }

    let on_submit = {
        let board = board.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            if board.joining.get_untracked() {
                return;
            }

            let room_id = match parse_room_id_input(&board.join_input.get_untracked()) {
                Ok(room_id) => room_id,
                Err(message) => {
                    board.join_error.set(Some(message));
                    return;
                }
            };

            board.join_error.set(None);
            board.joining.set(true);

            let navigate = navigate.clone();
            let board = board.clone();
            spawn_local(async move {
                match join_room_request(room_id).await {
                    Ok(_) => navigate(&room_route(room_id), Default::default()),
                    Err(message)
                        if message.contains("membership is already joined")
                            || message.contains("membership is already pending")
                            || message.contains("membership is pending approval") =>
                    {
                        navigate(&room_route(room_id), Default::default());
                    }
                    Err(message) => board.join_error.set(Some(message)),
                }
                board.joining.set(false);
            });
        }
    };

    let actions = RoomsBoardActions {
        request_leave: {
            let board = board.clone();
            Callback::new(move |room: JoinedRoomSummary| board.request_leave(room))
        },
        request_archive: {
            let board = board.clone();
            Callback::new(move |room: JoinedRoomSummary| board.request_archive(room))
        },
        cancel_action: {
            let board = board.clone();
            Callback::new(move |_| board.cancel_action())
        },
        confirm_action: {
            let board = board.clone();
            let refresh_rooms = refresh_rooms.clone();
            Callback::new(move |_| {
                let Some(dialog) = board.action_dialog.get_untracked() else {
                    return;
                };

                let room_id = dialog.room().room.id.into_inner();

                board.action_error.set(None);
                board.action_busy_room_id.set(Some(room_id));
                let board = board.clone();
                let refresh_rooms = refresh_rooms.clone();

                spawn_local(async move {
                    let result = match dialog {
                        RoomsActionDialog::Leave(_) => {
                            leave_room_request(room_id).await.map(|_| ())
                        }
                        RoomsActionDialog::Archive(_) => {
                            archive_room_request(room_id).await.map(|_| ())
                        }
                    };

                    match result {
                        Ok(()) => {
                            board.action_dialog.set(None);
                            refresh_rooms.run(());
                        }
                        Err(message) => board.action_error.set(Some(message)),
                    }

                    board.action_busy_room_id.set(None);
                });
            })
        },
    };

    view! { <form on:submit=on_submit>{rooms_page_content(board, actions)}</form> }
}
