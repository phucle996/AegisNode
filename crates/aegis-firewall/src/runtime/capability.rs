// Host Capability Detector kiểm tra khả năng tương thích môi trường Linux
// Phát hiện sự tồn tại của nft binary, kernel nftables support, IPv6 support và quyền hạn thực thi

use std::sync::Arc;

use aegis_core::Result;
use serde::{Deserialize, Serialize};

use super::process_runner::{ProcessRequest, ProcessRunner};

/// Báo cáo khả năng hoạt động của nftables trên host Linux
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NftCapabilityReport {
    pub nft_installed: bool,
    pub nft_version: String,
    pub has_permissions: bool,
    pub kernel_support: bool,
    pub ipv6_support: bool,
}

/// Detector phát hiện khả năng tương thích nftables
pub struct CapabilityDetector {
    runner: Arc<dyn ProcessRunner>,
}

impl CapabilityDetector {
    pub fn new(runner: Arc<dyn ProcessRunner>) -> Self {
        Self { runner }
    }

    /// Đánh giá toàn bộ khả năng tương thích môi trường hệ thống
    pub async fn detect(&self) -> Result<NftCapabilityReport> {
        let mut report = NftCapabilityReport {
            nft_installed: false,
            nft_version: String::new(),
            has_permissions: false,
            kernel_support: false,
            ipv6_support: false,
        };

        // 1. Kiểm tra sự tồn tại của binary 'nft' và phiên bản
        let version_req = ProcessRequest::new("nft", vec!["--version".to_string()]);
        if let Ok(output) = self.runner.run(version_req).await {
            if output.is_success() {
                report.nft_installed = true;
                report.nft_version = output.stdout.trim().to_string();
            }
        }

        if !report.nft_installed {
            return Ok(report);
        }

        // 2. Kiểm tra quyền hạn và kernel nftables support bằng câu lệnh syntax check rỗng
        let check_req = ProcessRequest::new("nft", vec!["list".to_string(), "tables".to_string()]);
        if let Ok(output) = self.runner.run(check_req).await {
            if output.is_success() {
                report.has_permissions = true;
                report.kernel_support = true;
            } else if output.stderr.contains("Permission denied")
                || output.stderr.contains("Operation not permitted")
            {
                report.kernel_support = true;
                report.has_permissions = false;
            }
        }

        // 3. Kiểm tra IPv6 Kernel support
        report.ipv6_support = std::path::Path::new("/proc/sys/net/ipv6").exists();

        Ok(report)
    }
}
