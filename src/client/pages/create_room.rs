use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

use crate::client::{
    context::page_title::use_static_page_title,
    utils::rooms::{create_room_request, room_route, validate_room_name_input},
};

stylance::import_style!(style, "create_room.module.scss");

fn create_room_page_content(
    room_name: RwSignal<String>,
    submitting: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_submit: impl Fn(SubmitEvent) + 'static,
) -> impl IntoView {
    view! {
        <section class=format!("g-page g-page-shell {}", style::create_room_page)>
            <div class="g-page-meta">
                <a class="g-button-utility" href="/rooms">
                    "Back to rooms"
                </a>
            </div>

            <div class=style::create_room_layout>
                <section class=format!("g-panel g-panel-strong {}", style::hero_card)>
                    <p class="g-section-label">"New table"</p>
                    <h1 class="g-section-title">"Create a room and open the live feed"</h1>
                    <p class="g-section-summary">
                        "Start with a room name only. Once the room exists, approvals, kicks, and rolls all happen from the room page."
                    </p>
                    <dl class=style::hero_meta>
                        <div class=style::hero_meta_row>
                            <dt>"What happens next"</dt>
                            <dd>"You land in the room immediately as its admin."</dd>
                        </div>
                    </dl>
                </section>

                <section class=format!("g-panel g-panel-strong {}", style::form_card)>
                    <p class="g-section-label">"Create room"</p>
                    <h2 class=style::form_title>"Name the table"</h2>
                    <form class=style::form_grid on:submit=on_submit>
                        <label class="g-field-label" for="create-room-name">
                            "Room name"
                        </label>
                        <input
                            id="create-room-name"
                            class="g-text-input"
                            type="text"
                            maxlength="20"
                            placeholder="Moonlit Ledger"
                            prop:value=move || room_name.get()
                            on:input=move |event| {
                                error.set(None);
                                room_name.set(event_target_value(&event));
                            }
                        />
                        {move || {
                            error
                                .get()
                                .map(|message| view! { <p class=style::form_error>{message}</p> })
                        }}
                        <button
                            class="g-button-action"
                            type="submit"
                            prop:disabled=move || {
                                submitting.get() || room_name.get().trim().is_empty()
                            }
                        >
                            {move || if submitting.get() { "Creating..." } else { "Create room" }}
                        </button>
                    </form>
                </section>
            </div>
        </section>
    }
}

#[component]
pub fn CreateRoomPage() -> impl IntoView {
    use_static_page_title("Create Room");

    let navigate = use_navigate();
    let room_name = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if submitting.get_untracked() {
            return;
        }

        let name = match validate_room_name_input(&room_name.get_untracked()) {
            Ok(name) => name,
            Err(message) => {
                error.set(Some(message));
                return;
            }
        };

        error.set(None);
        submitting.set(true);

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_room_request(name).await {
                Ok(room) => navigate(&room_route(room.id.into_inner()), Default::default()),
                Err(message) => error.set(Some(message)),
            }
            submitting.set(false);
        });
    };

    view! { {create_room_page_content(room_name, submitting, error, on_submit)} }
}
