use leptos::{ev::Event, prelude::*};

use super::room_stubs::{join_target_from_input, room_route, room_summaries, RoomSummary};

stylance::import_style!(style, "rooms.module.scss");

fn live_user_count_label(room: &RoomSummary) -> String {
    let count = room.live_users.len();
    if count == 1 {
        "1 live in room".to_string()
    } else {
        format!("{count} live in room")
    }
}

fn live_user_preview(room: &RoomSummary) -> String {
    let names = room
        .live_users
        .iter()
        .map(|entry| entry.display_name.as_str())
        .collect::<Vec<_>>();

    match names.as_slice() {
        [] => "Nobody is seated yet.".to_string(),
        [one] => format!("{one} is already at the table."),
        [one, two] => format!("{one} and {two} are already at the table."),
        [one, two, rest @ ..] => {
            format!("{one}, {two}, and {} more are already at the table.", rest.len())
        }
    }
}

fn rooms_page_content(rooms: Vec<RoomSummary>, initial_join_input: &str) -> impl IntoView {
    let join_input = RwSignal::new(initial_join_input.to_string());

    view! {
        <section class="g-page g-page-shell">
            <section class=format!("g-panel g-panel-strong {}", style::launch_panel)>
                <div class=style::launch_header>
                    <p class="g-section-label">"Ledger stage"</p>
                    <h1 class="g-section-title">"Open a table or step back into an active room."</h1>
                    <p class="g-section-summary">
                        "Room creation, membership, and validation stay stubbed in this pass. The page is here to frame active tables without pretending the backend already exists."
                    </p>
                </div>

                <div class=style::launch_grid>
                    <article class=style::launch_card>
                        <p class="g-section-label">"Start a room"</p>
                        <h2 class=style::launch_card_title>"Create a room"</h2>
                        <p class=style::launch_card_summary>
                            "Keep the entry point visible so the rooms board feels complete, but leave the control honest until creation wiring lands."
                        </p>
                        <div class=style::launch_action_row>
                            <button class="g-button-action" type="button" disabled>
                                "Create a room"
                            </button>
                            <p class=style::launch_helper>
                                "Creation wiring arrives in a later pass. Nothing is persisted yet."
                            </p>
                        </div>
                    </article>

                    <article class=style::launch_card>
                        <p class="g-section-label">"Rejoin a table"</p>
                        <h2 class=style::launch_card_title>"Join by room ID"</h2>
                        <label class="g-field-label" for="rooms-join-room-id">
                            "Room ID"
                        </label>
                        <div class=style::join_row>
                            <input
                                id="rooms-join-room-id"
                                class="g-text-input"
                                type="text"
                                placeholder="copper-annex"
                                prop:value=move || join_input.get()
                                on:input=move |event: Event| join_input.set(event_target_value(&event))
                            />
                            <div class=style::join_action_slot>
                                {move || {
                                    match join_target_from_input(&join_input.get()) {
                                        Some(target) => {
                                            view! {
                                                <a class="g-button-action" href=target>
                                                    "Join room"
                                                </a>
                                            }
                                                .into_any()
                                        }
                                        None => {
                                            view! {
                                                <button class="g-button-action" type="button" disabled>
                                                    "Join room"
                                                </button>
                                            }
                                                .into_any()
                                        }
                                    }
                                }}
                            </div>
                        </div>
                        <p class=style::join_helper>
                            "Room validation and membership wiring are still pending. This action only exposes the room route for now."
                        </p>
                    </article>
                </div>
            </section>

            <section class=format!("g-panel g-panel-strong {}", style::joined_rooms_panel)>
                <div class=style::joined_rooms_header>
                    <p class="g-section-label">"Active tables"</p>
                    <h2 class="g-section-title">"Joined rooms"</h2>
                    <p class="g-section-summary">
                        "Each card is a current table snapshot: room note, who is live, and the latest ledger motion waiting behind the door."
                    </p>
                </div>

                {if rooms.is_empty() {
                    view! {
                        <div class=style::empty_state>
                            <h3 class=style::empty_state_title>"No joined rooms yet."</h3>
                            <p class=style::empty_state_copy>
                                "Use the create or join controls above once room wiring is ready."
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
                                    let live_count = live_user_count_label(&room);
                                    let live_preview = live_user_preview(&room);
                                    let room_target = room_route(&room.room_id);

                                    view! {
                                        <article class=style::room_card>
                                            <div class=style::room_card_header>
                                                <div class=style::room_identity>
                                                    <p class="g-section-label">{room.room_note.clone()}</p>
                                                    <h3 class=style::room_title>{room.room_title.clone()}</h3>
                                                </div>
                                                <span class=style::room_id_badge>{room.room_id.clone()}</span>
                                            </div>

                                            <dl class=style::room_meta_list>
                                                <div class=style::room_meta_row>
                                                    <dt>"Presence"</dt>
                                                    <dd>{live_count}</dd>
                                                </div>
                                                <div class=style::room_meta_row>
                                                    <dt>"Who is here"</dt>
                                                    <dd>{live_preview}</dd>
                                                </div>
                                                <div class=style::room_meta_row>
                                                    <dt>"Latest motion"</dt>
                                                    <dd>{room.recent_activity_line.clone()}</dd>
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
                }}
            </section>
        </section>
    }
}

#[component]
pub fn RoomsPage() -> impl IntoView {
    rooms_page_content(room_summaries(), "")
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use leptos::prelude::*;

    use crate::client::pages::room_stubs::room_summaries;

    use super::{rooms_page_content, RoomSummary};

    fn render_rooms_page_html(rooms: Vec<RoomSummary>, join_input: &str) -> String {
        let owner = Owner::new();
        owner.set();

        view! { <>{rooms_page_content(rooms, join_input)}</> }.to_html()
    }

    #[test]
    fn rooms_page_renders_launch_and_join_sections() {
        let html = render_rooms_page_html(room_summaries(), "");

        assert!(html.contains("Create a room"));
        assert!(html.contains("Join by room ID"));
        assert!(html.contains("Joined rooms"));
        assert!(html.contains("Join room</button>"));
    }

    #[test]
    fn rooms_page_renders_joined_room_cards() {
        let html = render_rooms_page_html(room_summaries(), "");

        assert!(html.contains("Moonlit Ledger"));
        assert!(html.contains("moonlit-ledger"));
        assert!(html.contains("3 live in room"));
        assert!(html.contains("Aria Vale"));
        assert!(html.contains("Latest motion: Mira logged a ward pulse at 14 total."));
    }

    #[test]
    fn rooms_page_renders_empty_state_when_no_joined_rooms() {
        let html = render_rooms_page_html(Vec::new(), "");

        assert!(html.contains("No joined rooms yet."));
        assert!(html.contains("Use the create or join controls above once room wiring is ready."));
    }

    #[test]
    fn rooms_page_exposes_encoded_join_target_for_trimmed_room_id() {
        let html = render_rooms_page_html(room_summaries(), "  Table 7/West Wing  ");

        assert!(html.contains("href=\"/room/Table%207%2FWest%20Wing\""));
        assert!(html.contains(">Join room</a>"));
    }

    #[test]
    fn rooms_page_links_joined_room_cards_to_room_detail() {
        let html = render_rooms_page_html(room_summaries(), "");

        assert!(html.contains("href=\"/room/moonlit-ledger\""));
        assert!(html.contains("href=\"/room/copper-annex\""));
        assert!(html.contains("Enter room"));
    }
}
