//! W3C Distributed Tracing Context Propagation (Phase 26 Production Observability)
//! Cung cấp cấu trúc W3cTraceContext và các phương thức parse / format header `traceparent` phục vụ Distributed Tracing.

use aegis_core::AegisError;
use uuid::Uuid;

/// Đơn vị lưu trữ W3C Trace Context (W3cTraceContext)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct W3cTraceContext {
    /// Phiên bản W3C Spec (Mặc định: 00)
    pub version: String,
    /// ID duy nhất của toàn bộ cuộc gọi (32 hexadecimal characters)
    pub trace_id: String,
    /// ID duy nhất đại diện cho Span hiện tại (16 hexadecimal characters)
    pub parent_id: String,
    /// Cờ kiểm soát tracing flags (Mặc định: 01 - sampled)
    pub trace_flags: String,
}

impl W3cTraceContext {
    /// Khởi tạo một Trace Context mới ngẫu nhiên (Root Span)
    pub fn generate_new() -> Self {
        let trace_uuid = Uuid::new_v4().simple().to_string(); // 32 hex chars
        let parent_uuid = &Uuid::new_v4().simple().to_string()[..16]; // 16 hex chars

        Self {
            version: "00".to_string(),
            trace_id: trace_uuid,
            parent_id: parent_uuid.to_string(),
            trace_flags: "01".to_string(),
        }
    }

    /// Định dạng W3C Trace Context thành chuỗi header `traceparent` tiêu chuẩn
    /// Ví dụ: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
    pub fn format_traceparent(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.version, self.trace_id, self.parent_id, self.trace_flags
        )
    }

    /// Parse chuỗi header `traceparent` nhận từ HTTP Request / gRPC Metadata
    pub fn parse_traceparent(header_value: &str) -> Result<Self, AegisError> {
        let parts: Vec<&str> = header_value.trim().split('-').collect();

        if parts.len() != 4 {
            return Err(AegisError::Validation(format!(
                "Header traceparent W3C không đúng định dạng (Mong muốn 4 phần ngăn cách bởi dấu '-'): '{header_value}'"
            )));
        }

        if parts[0] != "00" {
            return Err(AegisError::Validation(format!(
                "Phiên bản W3C traceparent version '{}' không được hỗ trợ",
                parts[0]
            )));
        }

        if parts[1].len() != 32 {
            return Err(AegisError::Validation(format!(
                "Độ dài trace_id trong traceparent phải đúng 32 ký tự hex (Nhận: {})",
                parts[1].len()
            )));
        }

        if parts[2].len() != 16 {
            return Err(AegisError::Validation(format!(
                "Độ dài parent_id trong traceparent phải đúng 16 ký tự hex (Nhận: {})",
                parts[2].len()
            )));
        }

        Ok(Self {
            version: parts[0].to_string(),
            trace_id: parts[1].to_string(),
            parent_id: parts[2].to_string(),
            trace_flags: parts[3].to_string(),
        })
    }
}
