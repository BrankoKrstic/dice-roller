use leptos::{prelude::*, task::spawn_local};

use crate::client::{
    components::dark_mode_toggle::DarkModeToggle,
    context::{
        auth::{logout, use_auth_context},
        page_title::use_page_title_context,
    },
};

stylance::import_style!(style, "nav_bar.module.scss");

#[component]
pub fn NavBar() -> impl IntoView {
    let auth = use_auth_context();
    let page_title = use_page_title_context();
    let menu_open = RwSignal::new(false);

    let toggle_menu = move |_| {
        menu_open.update(|open| *open = !*open);
    };

    view! {
        <header class=style::header>
            <div class=style::header_inner>
                <div class=style::header_shell>
                    <a class=style::brand_link href="/" on:click=move |_| menu_open.set(false)>
                        <span class=style::brand_mark>
                            <span class=style::brand_mark_text>"d20"</span>
                        </span>
                        <span class=style::brand_copy>
                            <span class="g-page-eyebrow">{move || page_title.get()}</span>
                            <span class=style::brand_text>"Dice Roller"</span>
                        </span>
                    </a>

                    <div class=style::header_controls>
                        <DarkModeToggle />
                        <button
                            class=style::menu_toggle
                            type="button"
                            aria-controls="primary-site-nav"
                            aria-expanded=move || if menu_open.get() { "true" } else { "false" }
                            on:click=toggle_menu
                        >
                            {move || if menu_open.get() { "Close" } else { "Menu" }}
                        </button>
                    </div>
                </div>

                <nav
                    id="primary-site-nav"
                    class=style::nav
                    class=(style::nav_open, move || menu_open.get())
                    aria-label="Main"
                >
                    <div class=style::nav_group>
                        <a class=style::nav_link href="/" on:click=move |_| menu_open.set(false)>
                            <span class=style::nav_link_label>"Roller"</span>
                            <span class=style::nav_link_hint>"Local ledger"</span>
                        </a>
                        <a
                            class=style::nav_link
                            href="/chance"
                            on:click=move |_| menu_open.set(false)
                        >
                            <span class=style::nav_link_label>"Chance"</span>
                            <span class=style::nav_link_hint>"Simulation ledger"</span>
                        </a>
                        <a
                            class=style::nav_link
                            href="/reference"
                            on:click=move |_| menu_open.set(false)
                        >
                            <span class=style::nav_link_label>"Reference"</span>
                            <span class=style::nav_link_hint>"Dice notation guide"</span>
                        </a>
                    </div>

                    <div class=style::nav_meta>
                        {move || {
                            if auth.loading.get() {
                                view! {
                                    <span class=style::nav_status>"Checking table access"</span>
                                }
                                    .into_any()
                            } else if let Some(user) = auth.user.get() {
                                view! {
                                    <>
                                        <a
                                            class=format!(
                                                "{} {}",
                                                style::nav_link,
                                                style::nav_meta_item,
                                            )
                                            href="/rooms"
                                            on:click=move |_| menu_open.set(false)
                                        >
                                            <span class=style::nav_link_label>"Rooms"</span>
                                            <span class=style::nav_link_hint>"Shared tables"</span>
                                        </a>
                                        <span class=format!(
                                            "{} {}",
                                            style::nav_status,
                                            style::nav_meta_item,
                                        )>
                                            {format!("Signed in as {}", user.username.as_str())}
                                        </span>
                                        <button
                                            class=format!(
                                                "g-button-utility {} {}",
                                                style::nav_action,
                                                style::nav_meta_item,
                                            )
                                            type="button"
                                            on:click=move |_| {
                                                menu_open.set(false);
                                                spawn_local(logout());
                                            }
                                        >
                                            "Logout"
                                        </button>
                                    </>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <>
                                        <a
                                            class=format!(
                                                "g-button-action {} {}",
                                                style::nav_action,
                                                style::nav_meta_item,
                                            )
                                            href="/login"
                                            on:click=move |_| menu_open.set(false)
                                        >
                                            <span class=style::nav_link_label>"Login"</span>
                                        </a>
                                        <a
                                            class=format!(
                                                "g-button-action {} {}",
                                                style::nav_action,
                                                style::nav_meta_item,
                                            )
                                            href="/register"
                                            on:click=move |_| menu_open.set(false)
                                        >
                                            "Register"
                                        </a>
                                    </>
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </nav>
            </div>
        </header>
    }
}
