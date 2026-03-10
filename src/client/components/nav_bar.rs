use leptos::{prelude::*, task::spawn_local};

use crate::client::{
    components::dark_mode_toggle::DarkModeToggle,
    context::auth::{logout, use_auth_context},
};

stylance::import_style!(style, "nav_bar.module.scss");

#[component]
pub fn NavBar() -> impl IntoView {
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

                    {move || {
                        if auth.loading.get() {
                            view! { <span class=style::nav_user>"..."</span> }.into_any()
                        } else if let Some(user) = auth.user.get() {
                            view! {
                                <>
                                    <a class=style::nav_link href="/rooms">
                                        "Rooms"
                                    </a>
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
                <DarkModeToggle />

            </div>
        </header>
    }
}
