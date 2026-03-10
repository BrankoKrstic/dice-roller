use dice_roller::shared::data::user::AuthContext;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use dice_roller::server::api::create_router;
    use dice_roller::server::{api::{AppState}, db::Db};
    use dice_roller::{app::shell, server::services::auth::AuthService};
    use dice_roller::client::App;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let _ = dotenvy::dotenv();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let router = create_router().await;

    let db = Db::from_env().await.unwrap();
	let auth = AuthService::from_env(db).await.unwrap();

    let state = AppState {
        leptos_options: leptos_options.clone(),
        auth: auth.clone()
    };


    let app = Router::<AppState>::new()
        .nest("/api", router)
        .leptos_routes(&state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
