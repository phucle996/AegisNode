// Host Capability Detector kiểm tra khả năng tương thích môi trường Linux
// Phát hiện sự tồn tại của nft binary, kernel nftables support, IPv6 support và quyền hạn thực thi

use std::sync::Arc;

use aegis_core::Result;
use serde::{Deserialize, Serialize};

use crate::process_runner::{ProcessRequest, ProcessRunner};

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

        // 1. Kiểm tra nft binary & version
        let req = ProcessRequest::new("nft", vec!["--version".to_string()]);
        if let Ok(out) = self.runner.run(req).await {
            if out.is_success() {
                report.nft_installed = true;
                report.nft_version = out.stdout.trim().to_string();
                report.kernel_support = true;
            }
        }

        // 2. Kiểm tra quyền hạn (Thử list ruleset hoặc nft --check)
        if report.nft_installed {
            let req_perm =
                ProcessRequest::new("nft", vec!["list".to_string(), "ruleset".to_string()]);
            if let Ok(out) = self.runner.run(req_perm).await {
                if out.is_success() {
                    report.has_permissions = true;
                }
            }
        }

        // 3. Kiểm tra hỗ trợ IPv6
        let req_v6 = ProcessRequest::new("ip", vec!["-6".to_string(), "addr".to_string()]);
        if let Ok(out) = self.runner.run(req_v6).await {
            if out.is_success() && !out.stdout.is_empty() {
                report.ipv6_support = true;
            }
        }

        Ok(report)
    }
}
