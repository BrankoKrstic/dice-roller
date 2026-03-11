use leptos::prelude::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "ssr")]
    #[test]
    fn not_found_page_offers_a_recovery_surface() {
        use leptos::prelude::*;

        let rendered = view! { <super::NotFoundPage /> };
        let html = rendered.to_html();

        assert!(html.contains("Route Lost"));
        assert!(html.contains("Return to Roller"));
    }
}
