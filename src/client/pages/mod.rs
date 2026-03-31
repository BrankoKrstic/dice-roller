use leptos::prelude::*;
use leptos_router::{
    components::{ProtectedRoute, Route, Router, Routes},
    path,
};

use crate::client::{
    context::auth::use_auth_context,
    pages::{
        create_room::CreateRoomPage, home::HomePage, login::LoginPage, not_found::NotFoundPage,
        reference::ReferencePage, register::RegisterPage, room::RoomPage, rooms::RoomsPage,
        stats::StatsPage,
    },
};

mod create_room;
mod home;
mod login;
mod not_found;
mod reference;
mod reference_content;
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
            <main class="g-app-main">
                <Routes fallback=NotFoundPage>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/chance") view=StatsPage />
                    <Route path=path!("/reference") view=ReferencePage />
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
                    <ProtectedRoute
                        path=path!("/rooms/create")
                        view=CreateRoomPage
                        condition=move || {
                            if auth.loading.get() { None } else { Some(auth.user.get().is_some()) }
                        }
                        redirect_path=|| "/"
                    />
                    <ProtectedRoute
                        path=path!("/room/:roomId")
                        view=RoomPage
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
