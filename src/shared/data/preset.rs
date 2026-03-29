use serde::{Deserialize, Serialize};

use crate::shared::data::user::UserId;

#[derive(Serialize, Deserialize)]
pub struct PresetRequest {
    pub name: String,
    pub expr: String,
}

#[derive(Serialize, Deserialize)]
pub struct PresetId(pub i64);

#[derive(Serialize, Deserialize)]
pub struct Preset {
    pub id: PresetId,
    pub name: String,
    pub expr: String,
}
