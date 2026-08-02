//! RBAC & Approval Models (Phase 21 Authorization Engine)
//! Định nghĩa các kiểu dữ liệu Role, Permission, RiskLevel, UserSubject, Claims và ApprovalRecord.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT Claims Payload chứa danh sách Roles và Permissions được inject từ Linux PAM Auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - Tên tài khoản người dùng Linux OS
    pub sub: String,
    /// Danh sách Roles của người dùng
    pub roles: Vec<Role>,
    /// Danh sách Permissions dạng `object:behavior` (VD: "firewall:apply", "systemd:restart", "*:*")
    pub permissions: Vec<String>,
    /// Thời điểm hết hạn của Token (Epoch timestamp seconds)
    pub exp: u64,
    /// Thời điểm khởi tạo Token (Epoch timestamp seconds)
    pub iat: u64,
}

impl Claims {
    /// Kiểm tra người dùng có quyền thực hiện hành động trên đối tượng hay không (`object:behavior`)
    /// Hỗ trợ kiểm tra wildcard `*:*` (toàn quyền) hoặc `resource:*` (toàn quyền trên resource)
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        let target = format!("{}:{}", resource, action);
        let resource_wildcard = format!("{}:*", resource);

        for perm in &self.permissions {
            if perm == "*:*" || perm == &target || perm == &resource_wildcard {
                return true;
            }
        }
        false
    }
}

/// Danh mục 5 Roles phân quyền chính trong AegisNode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Quyền chỉ đọc thông tin (View only)
    Viewer,
    /// Quyền vận hành hệ thống thông thường (Operations)
    Operator,
    /// Quyền quản trị An toàn & Bảo mật (Security Admin)
    SecurityAdmin,
    /// Quyền quản trị Nền tảng & Hạ tầng (Platform Admin)
    PlatformAdmin,
    /// Quyền theo dõi & Kiểm toán (Auditor)
    Auditor,
}

impl Role {
    /// Trả về danh sách các Permissions được gán mặc định cho Role
    pub fn default_permissions(&self) -> Vec<Permission> {
        match self {
            Self::Viewer => vec![
                Permission::NodesRead,
                Permission::FirewallRead,
                Permission::NetworkRead,
                Permission::ServiceRead,
                Permission::AuditRead,
            ],
            Self::Operator => vec![
                Permission::NodesRead,
                Permission::FirewallRead,
                Permission::FirewallWrite,
                Permission::NetworkRead,
                Permission::NetworkWrite,
                Permission::ServiceRead,
                Permission::ServiceRestart,
                Permission::AuditRead,
            ],
            Self::SecurityAdmin => vec![
                Permission::NodesRead,
                Permission::FirewallRead,
                Permission::FirewallWrite,
                Permission::FirewallApply,
                Permission::ChangePlanApprove,
                Permission::AuditRead,
            ],
            Self::PlatformAdmin => vec![
                Permission::NodesRead,
                Permission::FirewallRead,
                Permission::FirewallWrite,
                Permission::FirewallApply,
                Permission::NetworkRead,
                Permission::NetworkWrite,
                Permission::ServiceRead,
                Permission::ServiceRestart,
                Permission::ChangePlanApprove,
                Permission::RolloutManage,
                Permission::AuditRead,
                Permission::AdminManage,
            ],
            Self::Auditor => vec![
                Permission::NodesRead,
                Permission::FirewallRead,
                Permission::NetworkRead,
                Permission::ServiceRead,
                Permission::AuditRead,
            ],
        }
    }
}

/// Danh mục 12 Quyền hạn chi tiết (Granular Permissions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Quyền đọc thông tin danh sách Node
    NodesRead,
    /// Quyền đọc cấu hình Firewall Policy
    FirewallRead,
    /// Quyền chỉnh sửa cấu hình Firewall Policy
    FirewallWrite,
    /// Quyền thực thi Safe Apply Firewall Policy
    FirewallApply,
    /// Quyền xem cấu hình mạng và interface
    NetworkRead,
    /// Quyền thay đổi cấu hình mạng và interface
    NetworkWrite,
    /// Quyền xem danh sách dịch vụ Systemd
    ServiceRead,
    /// Quyền điều khiển và khởi động lại dịch vụ Systemd
    ServiceRestart,
    /// Quyền phê duyệt Change Plan có rủi ro
    ChangePlanApprove,
    /// Quyền quản lý tiến trình Multi-Node Rollout
    RolloutManage,
    /// Quyền xem nhật ký kiểm toán Audit Logs
    AuditRead,
    /// Quyền quản trị tài khoản người dùng và hệ thống
    AdminManage,
}

/// Mức độ rủi ro của Change Plan (Risk Tiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Rủi ro thấp - Không yêu cầu duyệt
    Low,
    /// Rủi ro trung bình - Yêu cầu 1 Operator duyệt
    Medium,
    /// Rủi ro cao - Yêu cầu 1 SecurityAdmin duyệt
    High,
    /// Rủi ro cực cao - Yêu cầu 2-Person Approval (chữ ký từ 2 người khác nhau)
    Critical,
}

/// Đối tượng người dùng đã xác thực (UserSubject)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubject {
    /// ID duy nhất của người dùng
    pub id: String,
    /// Tên tài khoản người dùng
    pub username: String,
    /// Danh sách các Roles được gán
    pub roles: Vec<Role>,
}

/// Phạm vi áp dụng phân quyền (AccessScope)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessScope {
    /// Áp dụng toàn cục trên toàn hệ thống
    Global,
    /// Áp dụng cho một Nhóm Node cụ thể
    Group(String),
    /// Áp dụng cho một Node ID cụ thể
    NodeId(String),
    /// Áp dụng dựa theo bộ nhãn Labels (Key-Value)
    Labels(HashMap<String, String>),
}

/// Chữ ký Phê duyệt Change Plan (ApprovalRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    /// ID người phê duyệt (Approver ID)
    pub approver_id: String,
    /// Tên tài khoản người duyệt
    pub approver_username: String,
    /// Thời điểm thực hiện phê duyệt (ISO string)
    pub approved_at: String,
    /// Mức rủi ro tại thời điểm duyệt
    pub risk_level: RiskLevel,
    /// Ghi chú phê duyệt
    pub comments: Option<String>,
}
