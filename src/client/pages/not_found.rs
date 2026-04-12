use leptos::prelude::*;

use crate::client::context::page_title::use_static_page_title;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    use_static_page_title("Not Found");

    view! {
        <section class="g-page g-page-shell">
            <div class="g-panel g-panel-strong">
                <p class="g-section-label">"Recovery route"</p>
                <h1 class="g-section-title">"Route Lost"</h1>
                <p class="g-section-summary">
                    "That page is not on the current map. Head back to the main roller or jump into the simulation ledger instead of dead-ending here."
                </p>
                <div class="g-page-meta">
                    <a class="g-button-action" href="/">
                        "Return to Roller"
                    </a>
                    <a class="g-button-utility" href="/chance">
                        "Open Chance"
                    </a>
                </div>
            </div>
        </section>
    }
}
