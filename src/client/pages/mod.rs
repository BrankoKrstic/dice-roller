use leptos::prelude::*;
use leptos_router::{
    components::{ProtectedRoute, Route, Router, Routes},
    path,
};

use crate::client::{
    context::auth::use_auth_context,
    pages::{
        home::HomePage, login::LoginPage, not_found::NotFoundPage, register::RegisterPage,
        rooms::RoomsPage, stats::StatsPage,
    },
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
    let auth = use_auth_context();
    view! {
        <Router>
            <main class="app-main">
                <Routes fallback=NotFoundPage>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/chance") view=StatsPage />
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/register") view=RegisterPage />
                    <ProtectedRoute
                        path=path!("/rooms")
                        view=RoomsPage
                        condition=move || {
                            if auth.loading.get() { None } else { Some(auth.user.get().is_some()) }
                        }
                        redirect_path=|| "/"
                    />

                </Routes>
            </main>
        </Router>
    }
}
