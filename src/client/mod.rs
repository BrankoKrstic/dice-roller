pub mod components;
pub mod context;
pub mod pages;
pub mod utils;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};

use crate::client::{
    components::nav_bar::NavBar,
    context::theme::{provide_theme_context, use_theme_context},
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_theme_context();
    let theme_context = use_theme_context();
    view! {
        // content for this welcome page
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/dice-roller.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />
        <div class="app-wrapper" data-theme=move || theme_context.get().as_str()>
            <NavBar />
            <pages::AppRoutes />
        </div>
    }
}
