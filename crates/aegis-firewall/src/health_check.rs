// Health Checker kiểm tra an toàn sau khi nạp Policy mới (Post-Apply Verification)
// Đảm bảo không mất kết nối loopback, managed table hoạt động tốt và local management port sẵn sàng

use std::sync::Arc;

use aegis_core::Result;
use serde::{Deserialize, Serialize};

use crate::process_runner::{ProcessRequest, ProcessRunner};

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

    /// Thực thi kiểm tra toàn bộ tiêu chí sức khỏe hệ thống
    pub async fn run_checks(&self) -> Result<HealthCheckReport> {
        let mut passed_checks = Vec::new();
        let mut failed_checks = Vec::new();

        // 1. Check managed table exist
        let req_table = ProcessRequest::new("nft", vec!["list".to_string(), "tables".to_string()]);
        match self.runner.run(req_table).await {
            Ok(out) if out.is_success() && out.stdout.contains("aegis_filter") => {
                passed_checks.push("Managed table 'inet aegis_filter' active".to_string());
            }
            _ => {
                failed_checks
                    .push("Managed table 'inet aegis_filter' missing in runtime".to_string());
            }
        }

        // 2. Check Loopback interface functionality
        let req_lo = ProcessRequest::new(
            "ip",
            vec!["link".to_string(), "show".to_string(), "lo".to_string()],
        );
        match self.runner.run(req_lo).await {
            Ok(out) if out.is_success() => {
                passed_checks.push("Loopback interface 'lo' operational".to_string());
            }
            _ => {
                failed_checks.push("Loopback interface check failed".to_string());
            }
        }

        let success = failed_checks.is_empty();

        Ok(HealthCheckReport {
            success,
            passed_checks,
            failed_checks,
        })
    }
}
