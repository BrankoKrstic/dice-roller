use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{delete, get},
};

use crate::{
    server::{
        api::AppState,
        services::presets::{PresetErrorResponse, PresetService},
    },
    shared::data::{
        preset::{Preset, PresetId, PresetRequest},
        user::AuthUser,
    },
};

pub type PresetApiResult<T> = Result<T, (StatusCode, Json<PresetErrorResponse>)>;

#[axum::debug_handler]
async fn list_presets_handler(
    State(presets): State<PresetService>,
    Extension(user): Extension<AuthUser>,
) -> PresetApiResult<Json<Vec<Preset>>> {
    let presets = presets.list_presets(user.id).await?;
    Ok(Json(presets))
}

#[axum::debug_handler]
async fn create_preset_handler(
    State(presets): State<PresetService>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<PresetRequest>,
) -> PresetApiResult<Json<Preset>> {
    let preset = presets.create_preset(user.id, payload).await?;
    Ok(Json(preset))
}

#[axum::debug_handler]
async fn delete_preset_handler(
    State(presets): State<PresetService>,
    Extension(user): Extension<AuthUser>,
    Path(preset_id): Path<i64>,
) -> PresetApiResult<StatusCode> {
    presets.delete_preset(user.id, PresetId(preset_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn create_presets_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_presets_handler).post(create_preset_handler))
        .route("/{id}", delete(delete_preset_handler))
}
