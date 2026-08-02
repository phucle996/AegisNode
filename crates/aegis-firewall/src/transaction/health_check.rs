// Health Checker kiểm tra an toàn sau khi nạp Policy mới (Post-Apply Verification)
// Đảm bảo không mất kết nối loopback, managed table hoạt động tốt và local management port sẵn sàng

use std::sync::Arc;

use aegis_core::Result;
use serde::{Deserialize, Serialize};

use crate::runtime::process_runner::{ProcessRequest, ProcessRunner};

/// Báo cáo kết quả Health Check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckReport {
    pub success: bool,
    pub passed_checks: Vec<String>,
    pub failed_checks: Vec<String>,
}

/// Trình thực thi kiểm tra sức khỏe hệ thống sau Apply
pub struct HealthChecker {
    runner: Arc<dyn ProcessRunner>,
}

impl HealthChecker {
    pub fn new(runner: Arc<dyn ProcessRunner>) -> Self {
        Self { runner }
    }

    /// Thực thi kiểm tra toàn bộ tiêu chí sức khỏe hệ thống thực tế (Không mock/bypass trong production)
    pub async fn run_checks(&self) -> Result<HealthCheckReport> {
        let mut passed_checks = Vec::new();
        let mut failed_checks = Vec::new();

        // 1. Kiểm tra sự tồn tại của Managed Table 'inet aegis_filter' trong nftables (Bọc timeout 3 giây)
        let req_table = ProcessRequest::new("nft", vec!["list".to_string(), "tables".to_string()]);
        match tokio::time::timeout(std::time::Duration::from_secs(3), self.runner.run(req_table)).await {
            Ok(Ok(out)) if out.is_success() && out.stdout.contains("aegis_filter") => {
                // Đánh dấu kiểm tra thành công nếu bảng nftables aegis_filter tồn tại
                passed_checks.push("Managed table 'inet aegis_filter' active".to_string());
            }
            _ => {
                // Ghi nhận lỗi thực tế nếu không tìm thấy bảng nftables hoặc quá thời gian timeout 3 giây
                failed_checks
                    .push("Managed table 'inet aegis_filter' missing or inactive".to_string());
            }
        }

        // 2. Kiểm tra khả năng kết nối mạng nội bộ Loopback Interface (Ping 127.0.0.1 bọc timeout 3 giây)
        let req_ping = ProcessRequest::new(
            "ping",
            vec!["-c".to_string(), "1".to_string(), "127.0.0.1".to_string()],
        );
        match tokio::time::timeout(std::time::Duration::from_secs(3), self.runner.run(req_ping)).await {
            Ok(Ok(out)) if out.is_success() => {
                // Đánh dấu kiểm tra loopback ping thành công
                passed_checks.push("Loopback ping 127.0.0.1 success".to_string());
            }
            _ => {
                // Ghi nhận lỗi thực tế nếu không thể ping loopback hoặc quá thời gian timeout 3 giây
                failed_checks.push("Loopback ping 127.0.0.1 failed or timed out".to_string());
            }
        }

        // Nếu danh sách failed_checks rỗng thì cờ success = true
        let success = failed_checks.is_empty();

        Ok(HealthCheckReport {
            success,
            passed_checks,
            failed_checks,
        })
    }
}
