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
        scroll_lock::{provide_scroll_lock_context, use_scroll_lock_context},
        theme::{provide_theme_context, use_theme_context},
    },
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_theme_context();
    provide_auth_context();
    provide_scroll_lock_context();
    let theme_context = use_theme_context();
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
        <Title text="Dice Roller | Session Ledger" />
        <Body {..} class=body_class />
        <div class="g-app-wrapper" data-theme=theme_attr>
            <div class="g-app-shell">
                <NavBar />
                <pages::AppRoutes />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn global_styles_use_g_prefixed_shared_classes() {
        let styles = include_str!("../../style/main.scss");

        assert!(styles.contains(".g-page-shell-split"));
        assert!(styles.contains(".g-panel-strong"));
        assert!(styles.contains(".g-button-action"));
        assert!(styles.contains(".g-text-input"));
        assert!(styles.contains(".g-body-scroll-lock"));
        assert!(!styles.contains(".page-shell--split"));
        assert!(!styles.contains(".button-action"));
    }
}
