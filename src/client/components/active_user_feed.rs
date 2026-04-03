use leptos::prelude::*;

use crate::shared::data::{
    room::{RoomMembershipStatus, RoomRosterMember},
    user::UserId,
};

stylance::import_style!(style, "active_user_feed.module.scss");

#[component]
pub fn ActiveUserFeed(
    #[prop(into)] roster_members: Signal<Vec<RoomRosterMember>>,
    #[prop(into)] connected: Signal<bool>,
    can_manage_members: bool,
    #[prop(into)] busy_user_id: Signal<Option<i64>>,
    #[prop(into)] action_error: Signal<Option<String>>,
    #[prop(into)] on_allow: Callback<UserId>,
    #[prop(into)] on_request_kick: Callback<UserId>,
) -> impl IntoView {
    view! {
        <section class=format!("g-panel g-panel-strong {}", style::presence_card)>
            <div class=style::presence_header>
                <p class="g-section-label">"Live in room"</p>
                <h2 class=style::presence_title>"Room members"</h2>
                <p class=style::presence_summary>
                    {move || {
                        if connected.get() {
                            "Active room members."
                        } else {
                            "Roster is reconnecting. Existing room state stays visible while the stream retries."
                        }
                    }}
                </p>
            </div>

            {move || {
                action_error
                    .get()
                    .map(|message| view! { <p class=style::presence_feedback>{message}</p> })
            }}

            {move || {
                let members = roster_members.get();
                if members.is_empty() {
                    view! { <p class=style::presence_empty>"No visible room members yet."</p> }
                        .into_any()
                } else {
                    view! {
                        <div class=style::presence_scroll>
                            <ul class=style::presence_list>
                                <For
                                    each=move || roster_members.get()
                                    key=|member| member.user_id.into_inner()
                                    children=move |member| {
                                        let user_id = member.user_id.into_inner();
                                        let allow_user_id = member.user_id;
                                        let kick_user_id = member.user_id;

                                        view! {
                                            <li
                                                class=style::presence_row
                                                data-live=if member.is_live { "true" } else { "false" }
                                                data-tone=member.status.as_str()
                                            >
                                                <div class=style::presence_identity>
                                                    <strong class=style::presence_name>
                                                        {member.username.as_str().to_string()}
                                                    </strong>
                                                    <p class=style::presence_note>
                                                        {member_status_note(&member)}
                                                    </p>
                                                </div>
                                                <div class=style::presence_meta>
                                                    <div class=style::presence_badges>
                                                        <Show when=move || member.is_creator>
                                                            <span class=style::presence_badge>"GM"</span>
                                                        </Show>
                                                        <span class=style::presence_badge>
                                                            {if member.is_live { "Live" } else { "Offline" }}
                                                        </span>
                                                        <Show when=move || !member.is_creator>
                                                            <span class=style::presence_badge>
                                                                {membership_badge_label(member.status)}
                                                            </span>
                                                        </Show>
                                                    </div>
                                                    <Show when=move || can_manage_members && !member.is_creator>
                                                        <div class=style::presence_actions>
                                                            <Show when=move || {
                                                                matches!(
                                                                    member.status,
                                                                    RoomMembershipStatus::Pending | RoomMembershipStatus::Kicked
                                                                )
                                                            }>
                                                                <button
                                                                    class="g-button-utility"
                                                                    type="button"
                                                                    prop:disabled=move || {
                                                                        busy_user_id.get() == Some(user_id)
                                                                    }
                                                                    on:click={
                                                                        let on_allow = on_allow.clone();
                                                                        move |_| on_allow.run(allow_user_id)
                                                                    }
                                                                >
                                                                    {move || {
                                                                        if member.status == RoomMembershipStatus::Kicked {
                                                                            "Unkick"
                                                                        } else {
                                                                            "Admit"
                                                                        }
                                                                    }}
                                                                </button>
                                                            </Show>
                                                            <Show when=move || {
                                                                matches!(
                                                                    member.status,
                                                                    RoomMembershipStatus::Pending | RoomMembershipStatus::Joined
                                                                )
                                                            }>
                                                                <button
                                                                    class="g-button-ghost"
                                                                    type="button"
                                                                    prop:disabled=move || {
                                                                        busy_user_id.get() == Some(user_id)
                                                                    }
                                                                    on:click={
                                                                        let on_request_kick = on_request_kick.clone();
                                                                        move |_| on_request_kick.run(kick_user_id)
                                                                    }
                                                                >
                                                                    "Kick"
                                                                </button>
                                                            </Show>
                                                        </div>
                                                    </Show>
                                                </div>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </div>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

fn membership_badge_label(status: RoomMembershipStatus) -> &'static str {
    match status {
        RoomMembershipStatus::Pending => "Pending",
        RoomMembershipStatus::Joined => "Joined",
        RoomMembershipStatus::Kicked => "Kicked",
    }
}

fn member_status_note(member: &RoomRosterMember) -> String {
    if member.is_creator {
        if member.is_live {
            "Managing the table and connected right now.".to_string()
        } else {
            "Managing the table and currently away from the live stream.".to_string()
        }
    } else {
        match (member.status, member.is_live) {
            (RoomMembershipStatus::Pending, _) => {
                "Waiting for approval to join the shared ledger.".to_string()
            }
            (RoomMembershipStatus::Joined, true) => {
                "Joined and connected to the shared ledger right now.".to_string()
            }
            (RoomMembershipStatus::Joined, false) => {
                "Joined member who is currently offline.".to_string()
            }
            (RoomMembershipStatus::Kicked, _) => {
                "Removed from the room and ready to be reinstated.".to_string()
            }
        }
    }
}
