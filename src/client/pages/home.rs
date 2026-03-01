use leptos::prelude::*;

use crate::client::context::theme::{toggle_theme, use_theme_context, Theme};

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let theme = use_theme_context();

    view! {
        <div>
            <div>
                {move || match theme.get() {
                    Theme::Light => "LIGHT",
                    Theme::Dark => "DARK",
                }}
            </div>
            <button on:click=|_| toggle_theme()>"Toggle Theme"</button>
        </div>
    }
}
