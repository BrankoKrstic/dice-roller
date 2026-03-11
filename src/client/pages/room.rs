use leptos::prelude::*;

stylance::import_style!(style, "room.module.scss");

#[component]
pub fn RoomPage() -> impl IntoView {
    view! {
        <section class="g-page g-page-shell">
            <div class=style::room_hero>
                <p class="g-page-eyebrow">"Room detail"</p>
                <h1 class=style::room_title>"Room detail is not wired yet."</h1>
                <p class=style::room_summary>
                    "This route will eventually hold presence, shared notes, and live ledger state for a single table."
                </p>
                <div class="g-page-meta">
                    <a class="g-button-action" href="/rooms">
                        "Back to rooms"
                    </a>
                </div>
            </div>
        </section>
    }
}
