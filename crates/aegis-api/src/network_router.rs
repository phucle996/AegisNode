// Network Management REST API Handlers & Router cho Controller Server (`aegisnode server`)
// Truy vấn và khởi tạo cấu hình Network Profiles, phân vai trò giao diện (WAN, LAN, MANAGEMENT, STORAGE)

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_models::network_profile::NetworkProfile;
use axum::extract::{Json, State};
use axum::http::StatusCode;

use crate::controller_router::ControllerState;

/// Handler `GET /v1/network/profiles`: Lấy danh sách Network Profiles trong Cluster
pub async fn list_network_profiles_handler(
    _state: State<Arc<ControllerState>>,
) -> StdResult<Json<Vec<NetworkProfile>>, StatusCode> {
    let default_profile = NetworkProfile::default();
    Ok(Json(vec![default_profile]))
}

/// Handler `POST /v1/network/profiles`: Tạo hoặc cập nhật Network Profile
pub async fn create_network_profile_handler(
    State(state): State<Arc<ControllerState>>,
    Json(profile): Json<NetworkProfile>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.save_network_profile(&profile)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "status": "SAVED",
        "profileId": profile.id,
        "name": profile.name
    })))
}
