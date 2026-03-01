use leptos::prelude::*;

use crate::client::context::theme::{toggle_theme, use_theme_context, Theme};

stylance::import_style!(style, "nav_bar.module.scss");

#[component]
pub fn NavBar() -> impl IntoView {
    let theme = use_theme_context();

    view! {
        <header class=style::header>
            <div class=style::header_inner>
                <a class=style::brand href="/">
                    <span class=style::brand_mark>"d20"</span>
                    <span class=style::brand_text>"Dice Roller AI"</span>
                </a>

                <nav class=style::nav aria-label="Main">
                    <a class=style::nav_link href="/">
                        "Roller"
                    </a>
                    <a class=style::nav_link href="/chance">
                        "Chance"
                    </a>
                    <a class=style::nav_link href="/rooms">
                        "Rooms"
                    </a>
                </nav>

                <button
                    class=style::theme_toggle
                    on:click=move |_| toggle_theme()
                    aria-label=move || {
                        if theme.get() == Theme::Dark {
                            "Switch to light mode"
                        } else {
                            "Switch to dark mode"
                        }
                    }
                >
                    <span>
                        {move || { if theme.get() == Theme::Dark { "Dark" } else { "Light" } }}
                    </span>
                </button>
            </div>
        </header>
    }
}
