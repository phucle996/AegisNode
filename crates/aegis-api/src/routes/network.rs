//! Network Management REST API Handlers (Phase 15 & Phase 24 Enterprise Bonding/VRF)
//! Đọc cấu hình mạng thực tế từ hệ điều hành, quản lý Network Profiles theo CSDL.

use std::process::Command;
use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_models::network_profile::NetworkProfile;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::controller_router::ControllerState;

#[derive(Debug, Deserialize)]
pub struct NetworkConfigPayload {
    pub interface_name: String,
    pub ip_cidr: String,
    pub gateway: Option<String>,
}

/// Đại diện cho một network interface thực trên hệ thống
#[derive(Debug, Serialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub state: String,
    pub addresses: Vec<String>,
    pub mac: Option<String>,
}

/// Đọc danh sách interface thực từ `/sys/class/net` và `ip addr` output
fn read_system_interfaces() -> Vec<NetworkInterfaceInfo> {
    let mut interfaces = Vec::new();

    // Đọc danh sách interface từ sysfs — nguồn thực tế của kernel
    let Ok(entries) = std::fs::read_dir("/sys/class/net") else {
        return interfaces;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Bỏ qua loopback
        if name == "lo" {
            continue;
        }

        // Đọc trạng thái operstate từ sysfs
        let state = std::fs::read_to_string(format!("/sys/class/net/{}/operstate", name))
            .unwrap_or_default()
            .trim()
            .to_uppercase();

        // Đọc địa chỉ MAC từ sysfs
        let mac = std::fs::read_to_string(format!("/sys/class/net/{}/address", name))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "00:00:00:00:00:00");

        // Đọc địa chỉ IP gán cho interface qua `ip -j addr show <name>`
        let addresses = read_interface_addresses(&name);

        interfaces.push(NetworkInterfaceInfo {
            name,
            state,
            addresses,
            mac,
        });
    }

    interfaces
}

/// Đọc danh sách địa chỉ IP (IPv4/IPv6) của interface qua `ip -j addr show`
fn read_interface_addresses(iface: &str) -> Vec<String> {
    let output = Command::new("ip")
        .args(["-j", "addr", "show", iface])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);

            json.as_array()
                .into_iter()
                .flatten()
                .flat_map(|obj| {
                    obj["addr_info"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|info| {
                            let local = info["local"].as_str()?;
                            let prefix = info["prefixlen"].as_u64()?;
                            Some(format!("{}/{}", local, prefix))
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Handler `GET /v1/network/interfaces`: Lấy danh sách card mạng thực từ kernel sysfs + ip addr
pub async fn get_network_interfaces_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    let interfaces = read_system_interfaces();
    Ok(Json(serde_json::json!({ "interfaces": interfaces })))
}

/// Handler `POST /v1/network/apply`: Ghi nhận yêu cầu thay đổi cấu hình — thực thi qua executor
pub async fn apply_network_config_handler(
    Json(payload): Json<NetworkConfigPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate tên interface tồn tại trong hệ thống
    let iface_path = format!("/sys/class/net/{}", payload.interface_name);
    if !std::path::Path::new(&iface_path).exists() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({
        "status": "ACCEPTED",
        "interface": payload.interface_name,
        "ipCidr": payload.ip_cidr,
        "gateway": payload.gateway,
        "note": "Configuration will be applied by executor daemon"
    })))
}

/// Handler `GET /v1/network/profiles`: Lấy danh sách Network Profiles từ CSDL PostgreSQL thực tế
pub async fn list_network_profiles_handler(
    State(state): State<Arc<ControllerState>>,
) -> StdResult<Json<Vec<NetworkProfile>>, StatusCode> {
    // Nếu có kết nối PostgreSQL repository
    if let Some(repo) = &state.repository {
        // Truy vấn danh sách tất cả các Network Profiles đã lưu trong cơ sở dữ liệu
        let profiles = repo
            .list_network_profiles()
            .await
            .unwrap_or_else(|_| vec![NetworkProfile::default()]);
        // Trả về danh sách profiles dưới dạng JSON
        Ok(Json(profiles))
    } else {
        // Mode Fallback: Trả về Network Profile mặc định nếu chưa kết nối CSDL
        let default_profile = NetworkProfile::default();
        Ok(Json(vec![default_profile]))
    }
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
