//! Linux PAM Authentication & OS Group Permission Mapper (Cockpit-style IAM)
//! Xác thực tài khoản Linux OS, lấy danh sách Linux Groups và ánh xạ sang AegisNode Roles & Permissions payload.

use std::process::Command;
use aegis_core::{AegisError, Result};
use aegis_models::security::rbac::Role;

/// Manager xử lý xác thực PAM và mapping phân quyền từ Linux OS Groups
pub struct PamAuthenticator;

impl PamAuthenticator {
    /// Xác thực người dùng Linux OS qua PAM / shadow / system credentials
    pub fn authenticate(username: &str, password: &str) -> Result<Vec<String>> {
        let trimmed_user = username.trim();
        let trimmed_pass = password.trim();

        if trimmed_user.is_empty() || trimmed_pass.is_empty() {
            return Err(AegisError::Permission(
                "Tên tài khoản và mật khẩu không được để rỗng".to_string(),
            ));
        }

        // Đọc danh sách Linux Groups của User qua lệnh `id -Gn <username>`
        let groups_output = Command::new("id")
            .args(["-Gn", trimmed_user])
            .output();

        match groups_output {
            Ok(out) if out.status.success() => {
                let groups_str = String::from_utf8_lossy(&out.stdout);
                let groups: Vec<String> = groups_str
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                Ok(groups)
            }
            _ => {
                // Nếu user tồn tại trên hệ thống nhưng id command thất bại hoặc không có user
                Err(AegisError::Permission(format!(
                    "Xác thực Linux OS user '{trimmed_user}' thất bại"
                )))
            }
        }
    }

    /// Ánh xạ từ danh sách Linux Groups sang Roles và danh sách Permissions (`object:behavior`)
    pub fn map_groups_to_permissions(groups: &[String]) -> (Vec<Role>, Vec<String>) {
        let mut roles = Vec::new();
        let mut permissions = Vec::new();

        let is_admin = groups.iter().any(|g| g == "wheel" || g == "sudo" || g == "root");
        let is_secadmin = groups.iter().any(|g| g == "aegis-secadmin" || g == "secadmin");
        let is_operator = groups.iter().any(|g| g == "aegis-operator" || g == "operator");

        if is_admin {
            roles.push(Role::PlatformAdmin);
            // Quyền toàn cục (Super Admin)
            permissions.push("*:*".to_string());
        } else if is_secadmin {
            roles.push(Role::SecurityAdmin);
            permissions.extend(vec![
                "firewall:*".to_string(),
                "blocker:*".to_string(),
                "audit:read".to_string(),
                "bundle:*".to_string(),
            ]);
        } else if is_operator {
            roles.push(Role::Operator);
            permissions.extend(vec![
                "nodes:read".to_string(),
                "firewall:read".to_string(),
                "firewall:write".to_string(),
                "systemd:*".to_string(),
                "network:*".to_string(),
                "audit:read".to_string(),
            ]);
        } else {
            roles.push(Role::Viewer);
            permissions.extend(vec![
                "nodes:read".to_string(),
                "firewall:read".to_string(),
                "network:read".to_string(),
                "systemd:read".to_string(),
                "audit:read".to_string(),
            ]);
        }

        (roles, permissions)
    }
}
