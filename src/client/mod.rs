pub mod components;
pub mod context;
pub mod pages;
pub mod utils;
use leptos::prelude::*;
use leptos_meta::{Body, Stylesheet, Title, provide_meta_context};

use crate::client::{
    components::nav_bar::NavBar,
    context::{
        auth::provide_auth_context,
        page_title::{format_document_title, provide_page_title_context, use_page_title_context},
        scroll_lock::{provide_scroll_lock_context, use_scroll_lock_context},
        theme::{provide_theme_context, use_theme_context},
    },
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_theme_context();
    provide_auth_context();
    provide_page_title_context();
    provide_scroll_lock_context();
    let theme_context = use_theme_context();
    let page_title = use_page_title_context();
    let scroll_lock_context = use_scroll_lock_context();
    let theme_attr = move || theme_context.get().as_str().to_string();
    let body_class = move || {
        if scroll_lock_context.is_locked() {
            "g-body-scroll-lock".to_string()
        } else {
            String::new()
        }
    };

    view! {
        <Stylesheet id="leptos" href="/pkg/dice-roller.css" />
        <Title text=move || format_document_title(&page_title.get()) />
        <Body {..} class=body_class />
        <div class="g-app-wrapper" data-theme=theme_attr>
            <div class="g-app-shell">
                <NavBar />
                <pages::AppRoutes />
            </div>
        </div>
    }
}
