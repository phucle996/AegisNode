// Systemd Service Management Domain Models cho AegisNode Stage 2
// Định nghĩa các thao tác có định kiểu (Typed Operations), Trạng thái Unit và Journald Log DTO

use serde::{Deserialize, Serialize};

/// Các thao tác điều khiển dịch vụ có định kiểu (Typed Operations cho An toàn Bảo mật)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
}

/// DTO biểu diễn trạng thái của một Systemd Unit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceUnitStatus {
    pub name: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub description: String,
}

/// Request Payload yêu cầu thực thi thao tác điều khiển dịch vụ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOpRequest {
    pub unit_name: String,
    pub operation: ServiceOperation,
    pub reason: Option<String>,
}

/// Response Payload trả về kết quả thực thi thao tác dịch vụ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOpResult {
    pub success: bool,
    pub unit_name: String,
    pub operation: ServiceOperation,
    pub message: String,
    pub execution_time_ms: u64,
}

/// DTO bản tin nhật ký Journald Log Entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JournalLogEntry {
    pub timestamp: String,
    pub priority: String,
    pub unit: String,
    pub message: String,
}
