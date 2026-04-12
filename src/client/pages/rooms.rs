use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use crate::{
    client::{
        context::page_title::use_static_page_title,
        utils::rooms::{
            join_room_request, latest_roll_activity_line, list_joined_rooms_request,
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

fn rooms_page_content(
    rooms_state: Signal<JoinedRoomsState>,
    join_input: RwSignal<String>,
    joining: Signal<bool>,
    join_error: RwSignal<Option<String>>,
) -> impl IntoView {
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
                    <article class=style::launch_card>
                        <p class="g-section-label">"Start a room"</p>
                        <h2 class=style::launch_card_title>"Create a room"</h2>
                        <p class=style::launch_card_summary>"Spin up a new shared roll feed."</p>
                        <div class=style::launch_action_row>
                            <a id="rooms-create-room" class="g-button-action" href="/rooms/create">
                                "Create a room"
                            </a>
                        </div>
                    </article>

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
                                prop:value=move || join_input.get()
                                on:input=move |event| {
                                    join_error.set(None);
                                    join_input.set(event_target_value(&event));
                                }
                            />
                            <div class=style::join_action_slot>
                                <button
                                    id="rooms-join-submit"
                                    class="g-button-action"
                                    type="submit"
                                    disabled=move || joining.get()
                                >
                                    {move || if joining.get() { "Joining..." } else { "Join room" }}
                                </button>
                            </div>
                        </div>
                        <p class=style::join_helper>"Pending users wait there until admitted."</p>
                        {move || {
                            join_error
                                .get()
                                .map(|message| {
                                    view! { <p class=style::join_feedback>{message}</p> }
                                })
                        }}
                    </article>
                </div>
            </section>

            <section class=format!("g-panel g-panel-strong {}", style::joined_rooms_panel)>
                <div class=style::joined_rooms_header>
                    <p class="g-section-label">"Your rooms"</p>
                    <h2 class="g-section-title">"Joined rooms"</h2>
                    <p class="g-section-summary">"Rooms you've joined."</p>
                </div>

                {move || match rooms_state.get() {
                    JoinedRoomsState::Loading => {
                        view! {
                            <div class=style::empty_state>
                                <h3 class=style::empty_state_title>"Loading joined rooms..."</h3>
                                <p class=style::empty_state_copy>
                                    "Pulling your current room list from the server."
                                </p>
                            </div>
                        }
                            .into_any()
                    }
                    JoinedRoomsState::Error(message) => {
                        view! {
                            <div class=style::empty_state>
                                <h3 class=style::empty_state_title>
                                    "Could not load joined rooms."
                                </h3>
                                <p class=style::empty_state_copy>{message}</p>
                            </div>
                        }
                            .into_any()
                    }
                    JoinedRoomsState::Loaded(rooms) => {
                        if rooms.is_empty() {
                            view! {
                                <div class=style::empty_state>
                                    <h3 class=style::empty_state_title>"No joined rooms yet."</h3>
                                    <p class=style::empty_state_copy>
                                        "Create a room or request to join one by ID to build your active board."
                                    </p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class=style::rooms_grid>
                                    {rooms
                                        .into_iter()
                                        .map(|room| {
                                            let activity_line = latest_roll_activity_line(
                                                &room.latest_roll,
                                            );
                                            let room_target = room_route(room.room.id.into_inner());

                                            view! {
                                                <article class=style::room_card>
                                                    <div class=style::room_card_header>
                                                        <div class=style::room_identity>
                                                            <p class="g-section-label">
                                                                {if room.can_manage_members {
                                                                    "Admin".to_string()
                                                                } else {
                                                                    "Joined table".to_string()
                                                                }}
                                                            </p>
                                                            <h3 class=style::room_title>{room.room.name.clone()}</h3>
                                                        </div>
                                                        <span class=style::room_id_badge>
                                                            {format!("#{}", room.room.id.into_inner())}
                                                        </span>
                                                    </div>

                                                    <dl class=style::room_meta_list>
                                                        <div class=style::room_meta_row>
                                                            <dt>"Who is here"</dt>
                                                            <dd>
                                                                {if room.active_member_count == 0 {
                                                                    "Nobody is connected right now.".to_string()
                                                                } else {
                                                                    format!(
                                                                        "{} player{} connected to the room stream.",
                                                                        room.active_member_count,
                                                                        if room.active_member_count == 1 { "" } else { "s" },
                                                                    )
                                                                }}
                                                            </dd>
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
                                                    </div>
                                                </article>
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
        </section>
    }
}

#[component]
pub fn RoomsPage() -> impl IntoView {
    use_static_page_title("Rooms");

    let navigate = use_navigate();
    let join_input = RwSignal::new(String::new());
    let join_error = RwSignal::new(None::<String>);
    let joining = RwSignal::new(false);
    let rooms_state = RwSignal::new(JoinedRoomsState::Loading);

    Effect::new(move |_| {
        if !cfg!(feature = "hydrate") {
            return;
        }

        rooms_state.set(JoinedRoomsState::Loading);
        spawn_local(async move {
            match list_joined_rooms_request().await {
                Ok(rooms) => rooms_state.set(JoinedRoomsState::Loaded(rooms)),
                Err(message) => rooms_state.set(JoinedRoomsState::Error(message)),
            }
        });
    });

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if joining.get_untracked() {
            return;
        }

        let room_id = match parse_room_id_input(&join_input.get_untracked()) {
            Ok(room_id) => room_id,
            Err(message) => {
                join_error.set(Some(message));
                return;
            }
        };

        join_error.set(None);
        joining.set(true);

        let navigate = navigate.clone();
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
                Err(message) => join_error.set(Some(message)),
            }
            joining.set(false);
        });
    };

    view! {
        <form on:submit=on_submit>
            {rooms_page_content(rooms_state.into(), join_input, joining.into(), join_error)}
        </form>
    }
}
