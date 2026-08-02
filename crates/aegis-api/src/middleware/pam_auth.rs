// Linux PAM Authentication & OS Group Permission Mapper (Cockpit-style IAM)
// Xác thực tài khoản Linux OS bằng shadow password hash verification (pwhash), lấy danh sách Linux Groups và ánh xạ sang AegisNode Roles & Permissions payload.

use aegis_core::{AegisError, Result}; // Import các định nghĩa lỗi chuẩn của AegisNode
use aegis_models::security::rbac::Role; // Import enum Role bảo mật
use pwhash::unix;
use std::fs; // Thao tác đọc file hệ thống /etc/shadow
use std::process::Command; // Gọi lệnh hệ thống OS `id` // Thư viện verify chuẩn mã hóa Linux shadow password hash (SHA-512, SHA-256, Yescrypt, MD5)

/// Manager xử lý xác thực PAM và mapping phân quyền từ Linux OS Groups
pub struct PamAuthenticator;

impl PamAuthenticator {
    /// Xác thực người dùng Linux OS qua shadow/system credentials thực tế với password hash verification
    pub fn authenticate(username: &str, password: &str) -> Result<Vec<String>> {
        // Tối ưu loại bỏ khoảng trắng thừa ở 2 đầu username và password
        let trimmed_user = username.trim();
        let trimmed_pass = password.trim();

        // Kiểm tra tham số tài khoản và mật khẩu không được phép để rỗng
        if trimmed_user.is_empty() || trimmed_pass.is_empty() {
            return Err(AegisError::Permission(
                "Tên tài khoản và mật khẩu không được để rỗng".to_string(),
            ));
        }

        // 1. Đọc và xác minh hash mật khẩu thực tế từ /etc/shadow
        let shadow_content = fs::read_to_string("/etc/shadow").map_err(|e| {
            AegisError::Permission(format!(
                "Không thể truy cập /etc/shadow để xác thực tài khoản OS (cần quyền root): {e}"
            ))
        })?;

        // Tìm dòng chứa thông tin của username trong file /etc/shadow
        let user_line = shadow_content
            .lines()
            .find(|line| line.starts_with(&format!("{trimmed_user}:")))
            .ok_or_else(|| {
                AegisError::Permission(format!(
                    "Tài khoản '{trimmed_user}' không tồn tại trên hệ thống Linux"
                ))
            })?;

        // Tách các trường dữ liệu trong dòng shadow theo dấu hai chấm :
        let parts: Vec<&str> = user_line.split(':').collect();
        if parts.len() <= 1 {
            return Err(AegisError::Permission(format!(
                "Định dạng dòng shadow cho tài khoản '{trimmed_user}' không hợp lệ"
            )));
        }

        // Lấy chuỗi hash mật khẩu lưu trữ trong hệ thống
        let pass_hash = parts[1];

        // Nếu tài khoản bị khóa (! hoặc *) hoặc chưa thiết lập mật khẩu thì từ chối ngay lập tức
        if pass_hash.starts_with('!') || pass_hash.starts_with('*') || pass_hash.is_empty() {
            return Err(AegisError::Permission(format!(
                "Tài khoản hệ thống '{trimmed_user}' đang bị khóa hoặc chưa đặt mật khẩu"
            )));
        }

        // Thực hiện so sánh kiểm tra hash mật khẩu thực tế từ password truyền vào
        let is_password_valid = unix::verify(trimmed_pass, pass_hash);
        if !is_password_valid {
            return Err(AegisError::Permission(
                "Mật khẩu truy cập hệ thống Linux không chính xác".to_string(),
            ));
        }

        // 2. Đọc danh sách Linux Groups thực tế của User qua lệnh `id -Gn <username>`
        let groups_output = Command::new("id").args(["-Gn", trimmed_user]).output();

        match groups_output {
            Ok(out) if out.status.success() => {
                // Parse danh sách nhóm từ đầu ra stdout của lệnh id
                let groups_str = String::from_utf8_lossy(&out.stdout);
                let groups: Vec<String> = groups_str
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                // Trả về danh sách nhóm thực tế thu thập được
                Ok(groups)
            }
            _ => {
                // Trả về lỗi nếu không thể truy vấn nhóm của user từ OS
                Err(AegisError::Permission(format!(
                    "Xác thực Linux OS user '{trimmed_user}' thất bại khi đọc danh sách Linux Groups"
                )))
            }
        }
    }

    /// Ánh xạ từ Linux OS User sang Roles và Permissions (`object:behavior`)
    /// Mọi tài khoản hợp lệ trên Linux OS khi xác thực thành công đều được cấp toàn quyền Admin (*:*)
    pub fn map_groups_to_permissions(_groups: &[String]) -> (Vec<Role>, Vec<String>) {
        // Khởi tạo vector chứa danh sách Roles và Permissions
        let mut roles = Vec::new();
        let mut permissions = Vec::new();

        // Cấp Role PlatformAdmin cho bất kỳ tài khoản Linux OS nào xác thực thành công
        roles.push(Role::PlatformAdmin);
        // Cấp toàn quyền quản trị (*:*) không phân biệt nhóm
        permissions.push("*:*".to_string());

        (roles, permissions)
    }
}
