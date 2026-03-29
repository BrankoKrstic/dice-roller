use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use super::room_stubs::{
    RoomRosterEntry, RoomStub, build_local_room_roll, find_room_by_id, normalize_room_id_input,
};
use crate::{
    client::components::{roll_editor::RollEditor, roll_feed::RollFeed},
    dsl::parse_and_roll,
};

stylance::import_style!(style, "room.module.scss");

fn room_page_content(room: Option<RoomStub>, attempted_room_id: &str) -> impl IntoView {
    let attempted_room_id = attempted_room_id.trim().to_string();

    view! {
        <section class="g-page g-page-shell">
            <section class=format!("g-panel g-panel-strong {}", style::room_shell)>
                <div class="g-page-meta">
                    <a class="g-button-utility" href="/rooms">
                        "Back to rooms"
                    </a>
                </div>

                {match room {
                    Some(room) => {
                        let room_title = room.room_title.clone();
                        let room_id = room.room_id.clone();
                        let room_note = room.room_note.clone();
                        let live_users = room.live_users.clone();
                        let pending_users = room.pending_users.clone();
                        let feed = RwSignal::new(room.activity_feed.clone());
                        let load_older_rolls = Callback::new(|_| {});

                        let process_roll = move |expr: String| {
                            if let Ok(result) = parse_and_roll(&expr) {
                                let breakdown = result.to_string();
                                let new_roll =
                                    build_local_room_roll(&expr, result.total(), &breakdown);

                                feed.write().add_roll(new_roll);
                            }
                        };

                        view! {
                            <div class=format!("g-page-shell-split {}", style::room_layout)>
                                <section class=style::room_main>
                                    <section class=format!(
                                        "g-panel g-panel-strong {}",
                                        style::room_header,
                                    )>
                                        <div class=style::room_header_top>
                                            <p class="g-section-label">"Table ledger"</p>
                                            <h1 class=style::room_title>{room_title}</h1>
                                            <p class=style::room_summary>
                                                "Keep the next throw and the shared ledger in one place so everyone reads the same room state."
                                            </p>
                                        </div>

                                        <div class=style::room_header_meta>
                                            <span class=style::room_id_badge>{room_id}</span>
                                            <span class=style::room_note_badge>{room_note}</span>
                                        </div>
                                    </section>

                                    <RollEditor on_roll=process_roll />
                                    <RollFeed feed=feed loading_more=false load_older_rolls=load_older_rolls />
                                </section>

                                <aside class=style::room_rail>
                                    <section class=format!(
                                        "g-panel g-panel-strong {}",
                                        style::presence_card,
                                    )>
                                        <p class="g-section-label">"Live in room"</p>
                                        <h2 class=style::presence_title>"Players at the table"</h2>
                                        <p class=style::presence_summary>
                                            "Presence is stubbed for now, but the room already reads like a live shared table."
                                        </p>
                                        <ul class=style::roster_list>
                                            {live_users
                                                .into_iter()
                                                .map(|entry| {
                                                    view! { <RosterRow entry=entry tone="live" /> }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </section>

                                    <section class=format!("g-panel {}", style::pending_card)>
                                        <p class="g-section-label">"Waiting for approval"</p>
                                        <h2 class=style::presence_title>"Soft presence"</h2>
                                        <p class=style::presence_summary>
                                            "Keep pending players visible without turning the rail into a moderation dashboard."
                                        </p>

                                        {if pending_users.is_empty() {
                                            view! {
                                                <p class=style::presence_empty>
                                                    "No one is waiting for approval."
                                                </p>
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <ul class=style::roster_list>
                                                    {pending_users
                                                        .into_iter()
                                                        .map(|entry| {
                                                            view! { <RosterRow entry=entry tone="pending" /> }
                                                        })
                                                        .collect_view()}
                                                </ul>
                                            }
                                                .into_any()
                                        }}
                                    </section>
                                </aside>
                            </div>
                        }
                            .into_any()
                    }
                    None => {
                        view! {
                            <section class=style::room_main>
                                <section class=format!(
                                    "g-panel g-panel-strong {}",
                                    style::not_found_card,
                                )>
                                    <p class="g-section-label">"Room lookup"</p>
                                    <h1 class=style::room_title>"Room not found."</h1>
                                    <p class=style::room_summary>
                                        {format!(
                                            "No local room stub matches \"{}\" yet. Try another room ID or head back to the active tables board.",
                                            attempted_room_id
                                        )}
                                    </p>
                                </section>
                            </section>
                        }
                            .into_any()
                    }
                }}
            </section>
        </section>
    }
}

#[component]
fn RosterRow(entry: RoomRosterEntry, tone: &'static str) -> impl IntoView {
    let display_name = entry.display_name;
    let presence_note = entry.presence_note;
    let status_label = entry.status_label;
    let status_label_for_show = status_label.clone();

    view! {
        <li class=style::roster_row data-tone=tone>
            <div class=style::roster_identity>
                <strong class=style::roster_name>{display_name}</strong>
                <p class=style::roster_note>{presence_note}</p>
            </div>
            <Show when=move || status_label_for_show.is_some()>
                <span class=style::roster_status>{status_label.clone().unwrap_or_default()}</span>
            </Show>
        </li>
    }
}

#[component]
pub fn RoomPage() -> impl IntoView {
    let params = use_params_map();

    view! {
        {move || {
            let attempted_room_id = normalize_room_id_input(
                &params.get().get("roomId").unwrap_or_default(),
            );
            let room = find_room_by_id(&attempted_room_id);

            room_page_content(room, &attempted_room_id).into_any()
        }}
    }
}

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use leptos::prelude::*;

    use crate::client::pages::room_stubs::{find_room_by_id, normalize_room_id_input};

    use super::{room_page_content, style};

    fn render_room_page_html(room_id: &str) -> String {
        let owner = Owner::new();
        owner.set();

        let normalized_room_id = normalize_room_id_input(room_id);
        let room = find_room_by_id(&normalized_room_id);

        view! { <>{room_page_content(room, &normalized_room_id)}</> }.to_html()
    }

    #[test]
    fn room_page_renders_table_first_layout_for_known_room() {
        let html = render_room_page_html("moonlit-ledger");

        assert!(html.contains(style::room_shell));
        assert!(html.contains("Moonlit Ledger"));
        assert!(html.contains("moonlit-ledger"));
        assert!(html.contains("Active table"));
        assert!(html.contains("href=\"/rooms\""));
        assert!(html.contains("Live in room"));
        assert!(html.contains("Waiting for approval"));
        assert!(html.contains("Compose the next throw."));
        assert!(html.contains("Room Activity"));
        assert!(html.contains("Aria Vale"));
        assert!(html.contains("d20 + 4"));

        let header_index = html.find("Moonlit Ledger").expect("expected room header");
        let editor_index = html
            .find("Compose the next throw.")
            .expect("expected roll editor");
        let feed_index = html.find("Room Activity").expect("expected room activity");
        let rail_index = html.find("Live in room").expect("expected live roster");

        assert!(header_index < editor_index);
        assert!(editor_index < feed_index);
        assert!(feed_index < rail_index);

        let no_pending_html = render_room_page_html("north-gate-audit");
        assert!(no_pending_html.contains("No one is waiting for approval."));
    }

    #[test]
    fn room_page_renders_not_found_state_for_unknown_room() {
        let html = render_room_page_html("  unknown%20room  ");

        assert!(html.contains(style::room_shell));
        assert!(html.contains("Room not found."));
        assert!(html.contains("unknown room"));
        assert!(html.contains("href=\"/rooms\""));
    }

    #[test]
    fn room_page_styles_establish_split_layout() {
        let styles = include_str!("room.module.scss");

        assert!(styles.contains(".room-layout {"));
        assert!(styles.contains("display: grid;"));
        assert!(styles.contains("gap: var(--grid-gap);"));
    }
}
