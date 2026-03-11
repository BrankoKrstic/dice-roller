use leptos::prelude::*;

stylance::import_style!(style, "rooms.module.scss");

#[component]
pub fn RoomsPage() -> impl IntoView {
    view! {
        <section class="g-page g-page-shell">
            <div class="g-panel g-panel-strong">
                <p class="g-section-label">"Shared play"</p>
                <h1 class="g-section-title">"Rooms are staging."</h1>
                <p class="g-section-summary">
                    "The shell already speaks in room language. This route is the intentional preview surface while multiplayer wiring catches up."
                </p>
            </div>

            <section class="g-panel g-panel-strong">
                <p class="g-section-label">"What is already in place"</p>
                <div class=style::status_grid>
                    <div class=style::status_tile>
                        <strong>"Ledger shell"</strong>
                        <span>"Room-aware framing already anchors the product."</span>
                    </div>
                    <div class=style::status_tile>
                        <strong>"Protected route"</strong>
                        <span>"Access stays behind account-aware navigation."</span>
                    </div>
                    <div class=style::status_tile>
                        <strong>"Next step"</strong>
                        <span>"Presence, invites, and shared history sync."</span>
                    </div>
                </div>
            </section>
        </section>
    }
}
