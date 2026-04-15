use leptos::{prelude::*, task::spawn_local};
use web_sys::SubmitEvent;

use crate::{
    client::utils::{
        async_state::MutationState,
        rooms::{add_room_member_request, validate_username_input},
    },
    shared::data::room::RoomId,
};

stylance::import_style!(style, "add_room_member.module.scss");

#[component]
pub fn AddRoomMember(room_id: RoomId) -> impl IntoView {
    let username = RwSignal::new(String::new());
    let submit_state = RwSignal::new(MutationState::idle());

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if submit_state.get_untracked().is_pending() {
            return;
        }

        let username_value = match validate_username_input(&username.get_untracked()) {
            Ok(username_value) => username_value,
            Err(message) => {
                submit_state.set(MutationState::error(message));
                return;
            }
        };

        submit_state.set(MutationState::pending());

        spawn_local(async move {
            match add_room_member_request(room_id.into_inner(), &username_value).await {
                Ok(_) => {
                    username.set(String::new());
                    submit_state.set(MutationState::success());
                }
                Err(message) => submit_state.set(MutationState::error(message)),
            }
        });
    };

    view! {
        <section class=format!("g-panel g-panel-strong {}", style::add_member_card)>
            <div class=style::add_member_header>
                <p class="g-section-label">"Member controls"</p>
                <h2 class=style::add_member_title>"Add by username"</h2>
                <p class=style::add_member_summary>
                    "Adding a player by username joins them to the room immediately. Left, pending, and kicked members can be restored from here."
                </p>
            </div>

            <form class=style::add_member_form on:submit=on_submit>
                <label class="g-field-label" for="add-room-member-username">
                    "Username"
                </label>
                <input
                    id="add-room-member-username"
                    class="g-text-input"
                    type="text"
                    maxlength="20"
                    placeholder="tablemate"
                    prop:value=move || username.get()
                    on:input=move |event| {
                        submit_state.set(MutationState::idle());
                        username.set(event_target_value(&event));
                    }
                />
                {move || {
                    match submit_state.get() {
                        MutationState::Error(message) => {
                            Some(view! { <p class=style::add_member_feedback>{message}</p> })
                        }
                        MutationState::Idle | MutationState::Pending | MutationState::Success => {
                            None
                        }
                    }
                }}
                <button
                    class="g-button-action"
                    type="submit"
                    prop:disabled=move || {
                        matches!(submit_state.get(), MutationState::Pending)
                            || username.get().trim().is_empty()
                    }
                >
                    {move || match submit_state.get() {
                        MutationState::Pending => "Adding...",
                        MutationState::Idle
                        | MutationState::Success
                        | MutationState::Error(_) => "Add member",
                    }}
                </button>
            </form>
        </section>
    }
}
