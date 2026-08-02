//! Executor Protocol Definitions (Privilege Separation Phase 20)
//! Định nghĩa các Enum dạng Typed Request & Response cho IPC giữa non-root Agent và privileged Execd daemon.

use serde::{Deserialize, Serialize};

/// Enum định nghĩa các Yêu cầu thực thi đặc quyền (ExecRequest)
/// Loại bỏ hoàn toàn việc truyền raw shell string hay biến môi trường tùy ý
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ExecRequest {
    /// Đọc cấu hình nftables hiện tại của AegisNode
    InspectFirewall,

    /// Áp dụng Ruleset nftables mới với nội dung compiled và Hash kiểm tra
    ApplyFirewallRuleset {
        nft_content: String,
        expected_hash: String,
    },

    /// Tạo Snapshot lưu trữ cấu hình trước khi thay đổi
    CreateSnapshot {
        label: String,
    },

    /// Phục hồi Snapshot cũ theo Snapshot ID
    RollbackSnapshot {
        snapshot_id: String,
    },

    /// Thực hiện thao tác kiểm tra hoặc điều khiển Service systemd
    ServiceOperation {
        unit_name: String,
        action: String,
    },
}

/// Enum định nghĩa Kết quả phản hồi (ExecResponse) từ Execd
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ExecResponse {
    /// Thao tác thực thi đặc quyền thành công
    Success {
        details: String,
    },

    /// Thao tác thất bại với Mã lỗi và Mô tả chi tiết
    Failure {
        code: String,
        message: String,
    },

    /// Kết quả kiểm tra Ruleset nftables
    FirewallReport {
        ruleset_json: String,
    },
}
