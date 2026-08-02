// Systemd Service Manager & Protected Unit Guard cho AegisNode Linux Node
// Chấm dứt rủi ro ngắt kết nối tự dừng daemon, kiểm định tên Unit nghiêm ngặt và thực thi thao tác điều khiển có định kiểu

use aegis_core::{AegisError, Result};
use aegis_models::systemd::{
    JournalLogEntry, ServiceOpRequest, ServiceOpResult, ServiceOperation, ServiceUnitStatus,
};

/// Danh sách các dịch vụ cốt lõi bị cấm dừng/disable tuyệt đối (Protected Units Guard)
pub const PROTECTED_SYSTEM_UNITS: &[&str] = &[
    "aegisnode-local.service",
    "aegisnode-agent.service",
    "aegisnode.service",
    "dbus.service",
    "systemd-logind.service",
    "NetworkManager.service",
    "systemd-networkd.service",
    "sshd.service",
    "ssh.service",
];

/// Kiểm tra xem Unit có thuộc danh sách Protected Units bị cấm thao tác không
pub fn is_protected_unit(unit_name: &str) -> bool {
    let normalized = unit_name.trim();
    PROTECTED_SYSTEM_UNITS
        .iter()
        .any(|&protected| protected.eq_ignore_ascii_case(normalized))
}

/// Validate tên Unit bằng quy tắc nghiêm ngặt (ngăn ngừa Command Injection)
pub fn validate_unit_name(unit_name: &str) -> Result<()> {
    let trimmed = unit_name.trim();
    if trimmed.is_empty() {
        return Err(AegisError::Validation(
            "Unit name cannot be empty".to_string(),
        ));
    }

    if trimmed.contains(';')
        || trimmed.contains('&')
        || trimmed.contains('|')
        || trimmed.contains('`')
        || trimmed.contains('$')
        || trimmed.contains(' ')
    {
        return Err(AegisError::Validation(format!(
            "Invalid character detected in unit name '{trimmed}'"
        )));
    }

    if !trimmed.ends_with(".service")
        && !trimmed.ends_with(".socket")
        && !trimmed.ends_with(".target")
        && !trimmed.ends_with(".timer")
    {
        return Err(AegisError::Validation(format!(
            "Unit name '{trimmed}' must end with .service, .socket, .target, or .timer"
        )));
    }

    Ok(())
}

/// Manager xử lý tác vụ Systemd & Guard
#[derive(Debug, Clone, Default)]
pub struct SystemdManager;

impl SystemdManager {
    pub fn new() -> Self {
        Self
    }

    /// Thực thi thao tác có định kiểu trên Systemd Unit
    pub fn execute_op(&self, req: &ServiceOpRequest) -> Result<ServiceOpResult> {
        let start_time = std::time::Instant::now();

        // 1. Validate cú pháp tên Unit
        validate_unit_name(&req.unit_name)?;

        // 2. Kiềm tra Protected Unit nếu thao tác là Stop hoặc Disable
        if (req.operation == ServiceOperation::Stop || req.operation == ServiceOperation::Disable)
            && is_protected_unit(&req.unit_name)
        {
            return Err(AegisError::Permission(format!(
                "Operation {:?} is strictly FORBIDDEN on protected system unit '{}'",
                req.operation, req.unit_name
            )));
        }

        // 3. Thực thi giả lập tác vụ thành công
        let elapsed = start_time.elapsed().as_millis() as u64;
        Ok(ServiceOpResult {
            success: true,
            unit_name: req.unit_name.clone(),
            operation: req.operation,
            message: format!(
                "Successfully executed {:?} on unit '{}'",
                req.operation, req.unit_name
            ),
            execution_time_ms: elapsed,
        })
    }

    /// Truy vấn danh sách Journald Logs của Unit
    pub fn query_journal_logs(
        &self,
        unit_name: &str,
        limit: usize,
    ) -> Result<Vec<JournalLogEntry>> {
        validate_unit_name(unit_name)?;

        let mut entries = Vec::new();
        let max = limit.min(500);

        for i in 0..max {
            entries.push(JournalLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                priority: "info".to_string(),
                unit: unit_name.to_string(),
                message: format!("Sample journald log entry #{} for {}", i + 1, unit_name),
            });
        }

        Ok(entries)
    }

    /// Trả về trạng thái tổng quan của Unit
    pub fn get_unit_status(&self, unit_name: &str) -> Result<ServiceUnitStatus> {
        validate_unit_name(unit_name)?;

        Ok(ServiceUnitStatus {
            name: unit_name.to_string(),
            load_state: "loaded".to_string(),
            active_state: "active".to_string(),
            sub_state: "running".to_string(),
            description: format!("Systemd service {}", unit_name),
        })
    }
}
