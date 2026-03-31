use leptos::prelude::*;

use crate::shared::data::room::ActiveRoomMember;

stylance::import_style!(style, "active_user_feed.module.scss");

#[component]
pub fn ActiveUserFeed(
    #[prop(into)] active_members: Signal<Vec<ActiveRoomMember>>,
    creator_id: i64,
    #[prop(into)] connected: Signal<bool>,
) -> impl IntoView {
    view! {
        <section class=format!("g-panel g-panel-strong {}", style::presence_card)>
            <div class=style::presence_header>
                <p class="g-section-label">"Live in room"</p>
                <h2 class=style::presence_title>"Active user feed"</h2>
                <p class=style::presence_summary>
                    {move || {
                        if connected.get() {
                            "Presence is live over the room stream."
                        } else {
                            "Presence is reconnecting. Existing room state stays visible while the stream retries."
                        }
                    }}
                </p>
            </div>

            {move || {
                let members = active_members.get();
                if members.is_empty() {
                    view! {
                        <p class=style::presence_empty>
                            "No one is actively connected yet."
                        </p>
                    }
                        .into_any()
                } else {
                    view! {
                        <ul class=style::presence_list>
                            <For
                                each=move || active_members.get()
                                key=|member| member.user_id.into_inner()
                                children=move |member| {
                                    let user_id = member.user_id.into_inner();
                                    let badge = if user_id == creator_id { "GM" } else { "Live" };

                                    view! {
                                        <li class=style::presence_row>
                                            <div class=style::presence_identity>
                                                <strong class=style::presence_name>
                                                    {member.username.as_str().to_string()}
                                                </strong>
                                                <p class=style::presence_note>
                                                    {if user_id == creator_id {
                                                        "Managing the table and its approvals.".to_string()
                                                    } else {
                                                        "Connected to the shared ledger right now.".to_string()
                                                    }}
                                                </p>
                                            </div>
                                            <span class=style::presence_badge>{badge}</span>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}
