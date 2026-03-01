use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::client::{
    // context::theme::provide_theme_context,
    pages::{home::HomePage, not_found::NotFoundPage},
};

mod home;
mod login;
mod not_found;
mod register;
mod room;
mod rooms;
mod stats;

#[component]
pub fn AppRoutes() -> impl IntoView {
    // provide_theme_context();
    view! {
        <Router>
            <main class="app-main">
                <Routes fallback=NotFoundPage>
                    <Route path=path!("/") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}
