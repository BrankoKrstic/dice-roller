#![recursion_limit = "512"]
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use dice_roller::server::observability;
    use tracing::error;

    if let Err(error) = observability::init() {
        eprintln!("failed to initialize observability: {error}");
        std::process::exit(1);
    }

    if let Err(error) = run().await {
        error!(error = %error, "server startup failed");
        std::process::exit(1);
    }
}

#[cfg(feature = "ssr")]
async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::Router;
    use dice_roller::client::App;
    use dice_roller::server::api::{create_router, rooms::RoomLiveHub};
    use dice_roller::server::{api::AppState, db::Db, observability};
    use dice_roller::{
        app::shell,
        server::services::{auth::AuthService, presets::PresetService, rooms::RoomService},
    };
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower::ServiceBuilder;
    use tower_http::{
        LatencyUnit,
        request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
        sensitive_headers::SetSensitiveRequestHeadersLayer,
        trace::{DefaultOnFailure, TraceLayer},
    };
    use tracing::{Span, field, info};

    let _ = dotenvy::dotenv();
    info!("starting server bootstrap");

    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    info!(bind_address = %addr, "loaded server configuration");

    let routes = generate_route_list(App);
    info!(route_count = routes.len(), "generated leptos routes");

    let db = Db::from_env().await?;
    info!("database connection ready");
    let auth = AuthService::from_env(db.clone()).await?;
    info!("auth service ready");
    let presets = PresetService::from_env(db.clone()).await?;
    info!("preset service ready");
    let rooms = RoomService::from_env(db).await?;
    info!("room service ready");

    let room_live = RoomLiveHub::new();
    let router = create_router(auth.clone());
    let sensitive_headers = observability::sensitive_headers();
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
            let matched_path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(axum::extract::MatchedPath::as_str)
                .unwrap_or_else(|| request.uri().path());
            let request_id = request
                .extensions()
                .get::<tower_http::request_id::RequestId>()
                .and_then(|request_id| request_id.header_value().to_str().ok())
                .unwrap_or("-");

            tracing::info_span!(
                "http_request",
                request_id,
                method = %request.method(),
                matched_path,
                status = field::Empty,
                latency_ms = field::Empty,
                user_id = field::Empty,
            )
        })
        .on_response(
            |response: &axum::http::Response<axum::body::Body>,
             latency: std::time::Duration,
             span: &Span| {
                let latency_ms = latency.as_millis() as u64;
                span.record("status", field::display(response.status().as_u16()));
                span.record("latency_ms", field::display(latency_ms));

                tracing::info!(
                    parent: span,
                    status = response.status().as_u16(),
                    latency_ms,
                    "http request completed"
                );
            },
        )
        .on_failure(
            DefaultOnFailure::new()
                .level(tracing::Level::ERROR)
                .latency_unit(LatencyUnit::Millis),
        );
    let http_observability = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(SetSensitiveRequestHeadersLayer::new(sensitive_headers))
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::x_request_id());

    let state = AppState {
        leptos_options: leptos_options.clone(),
        auth: auth.clone(),
        presets: presets.clone(),
        rooms: rooms.clone(),
        room_live,
    };

    let app = Router::<AppState>::new()
        .nest("/api", router)
        .leptos_routes(&state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(http_observability)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(bind_address = %addr, "server listening");
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
