use leptos::prelude::*;

use crate::shared::data::{
    room::{RoomMemberSummary, RoomMembershipStatus},
    user::UserId,
};

stylance::import_style!(style, "room_member_manager.module.scss");

#[component]
pub fn RoomMemberManager(
    #[prop(into)] members: Signal<Vec<RoomMemberSummary>>,
    #[prop(into)] busy_user_id: Signal<Option<i64>>,
    #[prop(into)] action_error: Signal<Option<String>>,
    #[prop(into)] on_allow: Callback<UserId>,
    #[prop(into)] on_request_kick: Callback<RoomMemberSummary>,
) -> impl IntoView {
    view! {
        <section class=format!("g-panel g-panel-strong {}", style::manager_card)>
            <div class=style::manager_header>
                <p class="g-section-label">"Member controls"</p>
                <h2 class=style::manager_title>"Roster controls"</h2>
                <p class=style::manager_summary>
                    "Approve new players, remove joined users, and keep kicked users visible for quick reinstatement."
                </p>
            </div>

            {move || {
                action_error
                    .get()
                    .map(|message| view! { <p class=style::manager_feedback>{message}</p> })
            }}

            <MemberGroup
                title="Pending requests"
                summary="Requests land here until you admit or reject them."
                tone="pending"
                members=move || filter_members(&members.get(), RoomMembershipStatus::Pending)
                busy_user_id=busy_user_id
                on_allow=on_allow
                on_request_kick=on_request_kick
            />
            <MemberGroup
                title="Joined members"
                summary="Current room members remain kickable from the same rail."
                tone="joined"
                members=move || filter_members(&members.get(), RoomMembershipStatus::Joined)
                busy_user_id=busy_user_id
                on_allow=on_allow
                on_request_kick=on_request_kick
            />
            <MemberGroup
                title="Kicked members"
                summary="Kicked users stay visible so you can unkick them without losing context."
                tone="kicked"
                members=move || filter_members(&members.get(), RoomMembershipStatus::Kicked)
                busy_user_id=busy_user_id
                on_allow=on_allow
                on_request_kick=on_request_kick
            />
        </section>
    }
}

fn filter_members(
    members: &[RoomMemberSummary],
    status: RoomMembershipStatus,
) -> Vec<RoomMemberSummary> {
    members
        .iter()
        .filter(|member| member.status == status)
        .cloned()
        .collect()
}

#[component]
fn MemberGroup(
    title: &'static str,
    summary: &'static str,
    tone: &'static str,
    #[prop(into)] members: Signal<Vec<RoomMemberSummary>>,
    #[prop(into)] busy_user_id: Signal<Option<i64>>,
    #[prop(into)] on_allow: Callback<UserId>,
    #[prop(into)] on_request_kick: Callback<RoomMemberSummary>,
) -> impl IntoView {
    view! {
        <section class=style::member_group data-tone=tone>
            <div class=style::member_group_header>
                <h3 class=style::member_group_title>{title}</h3>
                <p class=style::member_group_summary>{summary}</p>
            </div>

            {move || {
                let group_members = members.get();
                if group_members.is_empty() {
                    view! { <p class=style::member_group_empty>"No members in this state."</p> }
                        .into_any()
                } else {
                    view! {
                        <ul class=style::member_list>
                            <For
                                each=move || members.get()
                                key=|member| member.user_id.into_inner()
                                children=move |member| {
                                    let user_id = member.user_id.into_inner();

                                    view! {
                                        <li class=style::member_row>
                                            <div class=style::member_identity>
                                                <strong class=style::member_name>
                                                    {member.username.as_str().to_string()}
                                                </strong>
                                                <p class=style::member_state_note>
                                                    {member_status_note(member.status)}
                                                </p>
                                            </div>
                                            <div class=style::member_actions>
                                                <Show
                                                    when=move || {
                                                        matches!(
                                                            member.status,
                                                            RoomMembershipStatus::Pending
                                                                | RoomMembershipStatus::Kicked
                                                        )
                                                    }
                                                >
                                                    <button
                                                        class="g-button-utility"
                                                        type="button"
                                                        prop:disabled=move || {
                                                            busy_user_id.get() == Some(user_id)
                                                        }
                                                        on:click={
                                                            let on_allow = on_allow.clone();
                                                            move |_| on_allow.run(member.user_id)
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
                                                <Show
                                                    when=move || {
                                                        matches!(
                                                            member.status,
                                                            RoomMembershipStatus::Pending
                                                                | RoomMembershipStatus::Joined
                                                        )
                                                    }
                                                >
                                                    <button
                                                        class="g-button-ghost"
                                                        type="button"
                                                        prop:disabled=move || {
                                                            busy_user_id.get() == Some(user_id)
                                                        }
                                                        on:click={
                                                            let on_request_kick = on_request_kick.clone();
                                                            let member = member.clone();
                                                            move |_| on_request_kick.run(member.clone())
                                                        }
                                                    >
                                                        "Kick"
                                                    </button>
                                                </Show>
                                            </div>
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

fn member_status_note(status: RoomMembershipStatus) -> &'static str {
    match status {
        RoomMembershipStatus::Pending => "Waiting for approval to join the live room.",
        RoomMembershipStatus::Joined => "Joined and allowed to read and roll in the room.",
        RoomMembershipStatus::Kicked => "Removed from the room but ready to be reinstated.",
    }
}
