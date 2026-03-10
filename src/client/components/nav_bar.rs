use leptos::{prelude::*, task::spawn_local};

use crate::client::context::{
    auth::{logout, use_auth_context},
    theme::{Theme, toggle_theme, use_theme_context},
};

stylance::import_style!(style, "nav_bar.module.scss");

#[component]
pub fn NavBar() -> impl IntoView {
    let theme = use_theme_context();
    let auth = use_auth_context();
    view! {
        <header class=style::header>
            <div class=style::header_inner>
                <a class=style::brand href="/">
                    <span class=style::brand_mark>"d20"</span>
                    <span class=style::brand_text>"Dice Roller"</span>
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
                    {move || {
                        if auth.loading.get() {
                            view! { <span class=style::nav_user>"..."</span> }.into_any()
                        } else if let Some(user) = auth.user.get() {
                            view! {
                                <>
                                    <span class=style::nav_user>
                                        {format!("Hi, {}", user.username.as_str())}
                                    </span>
                                    <button
                                        class=style::nav_button
                                        type="button"
                                        on:click=move |_| spawn_local(logout())
                                    >

                                        "Logout"
                                    </button>
                                </>
                            }
                                .into_any()
                        } else {
                            view! {
                                <>
                                    <a class=style::nav_link href="/login">
                                        "Login"
                                    </a>
                                    <a class=style::nav_link href="/register">
                                        "Register"
                                    </a>
                                </>
                            }
                                .into_any()
                        }
                    }}
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
