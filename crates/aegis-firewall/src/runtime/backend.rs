// Nftables Runtime Backend triển khai toàn bộ giao diện FirewallBackend
// Thực thi giao dịch an toàn (Transaction): Inspect -> Validate -> Compile -> Syntax Check -> Snapshot -> Apply -> Verify

use std::path::PathBuf;
use std::sync::Arc;

use aegis_core::{AegisError, ExecutionId, Result, SnapshotId};
use aegis_models::firewall::FirewallPolicy;
use aegis_policy::ValidationReport;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::process_runner::{ProcessRequest, ProcessRunner};
use crate::compiler::{CompiledFirewallPolicy, FirewallCompiler, NftablesCompiler};
use crate::transaction::snapshot::{FirewallSnapshot, SnapshotManager};

/// Báo cáo trạng thái Runtime của Firewall
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallState {
    pub managed_tables: Vec<String>,
    pub rules_count: usize,
    pub active_policy_hash: Option<String>,
}

/// Kết quả của thao tác Apply Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub execution_id: ExecutionId,
    pub snapshot_id: SnapshotId,
    pub applied: bool,
    pub syntax_check_passed: bool,
    pub validation_report: ValidationReport,
}

/// Trait định nghĩa Runtime Backend giao tiếp trực tiếp với nhân Linux Kernel
#[async_trait]
pub trait FirewallBackend: Send + Sync {
    /// Kiểm định cú pháp nftables script độc lập trước khi nạp
    async fn check_syntax(&self, nft_script: &str) -> Result<()>;

    /// Nạp kịch bản nftables script vào nhân kernel Linux
    async fn apply_ruleset(&self, nft_script: &str) -> Result<()>;

    /// Sao lưu trạng thái firewall hiện tại thành Snapshot
    async fn create_snapshot(&self, description: &str) -> Result<FirewallSnapshot>;

    /// Khôi phục trạng thái firewall từ Snapshot ID
    async fn rollback_to_snapshot(&self, snapshot_id: &SnapshotId) -> Result<()>;

    /// Đọc thông tin trạng thái runtime hiện tại
    async fn inspect_state(&self) -> Result<FirewallState>;

    async fn validate(&self, policy: &FirewallPolicy) -> Result<ValidationReport> {
        Ok(aegis_policy::PolicyValidator::validate(policy))
    }

    async fn compile(&self, policy: &FirewallPolicy) -> Result<CompiledFirewallPolicy> {
        let compiler = NftablesCompiler::new();
        compiler.compile(policy)
    }

    async fn snapshot(&self, description: &str) -> Result<FirewallSnapshot> {
        self.create_snapshot(description).await
    }

    async fn apply(&self, compiled: &CompiledFirewallPolicy) -> Result<()> {
        self.apply_ruleset(&compiled.nft_script).await
    }

    async fn rollback(&self, snapshot: &FirewallSnapshot) -> Result<()> {
        self.rollback_to_snapshot(&snapshot.snapshot_id).await
    }
}

/// Triển khai NftablesRuntimeBackend cho Linux Kernel nftables engine
pub struct NftablesRuntimeBackend {
    runner: Arc<dyn ProcessRunner>,
    snapshot_manager: Arc<SnapshotManager>,
    candidate_file_dir: PathBuf,
}

impl NftablesRuntimeBackend {
    pub fn new(
        runner: Arc<dyn ProcessRunner>,
        snapshot_manager: Arc<SnapshotManager>,
        candidate_file_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            runner,
            snapshot_manager,
            candidate_file_dir: candidate_file_dir.into(),
        }
    }

    /// Đọc toàn bộ ruleset hiện tại của kernel qua lệnh `nft list ruleset`
    pub async fn read_kernel_ruleset(&self) -> Result<String> {
        let req = ProcessRequest::new("nft", vec!["list".to_string(), "ruleset".to_string()]);
        let output = self.runner.run(req).await?;

        if output.is_success() {
            Ok(output.stdout)
        } else {
            Ok(String::new())
        }
    }

    /// Đọc ruleset thuộc quyền quản lý của AegisNode (`inet aegis_filter`)
    pub async fn read_managed_ruleset(&self) -> Result<String> {
        let req = ProcessRequest::new(
            "nft",
            vec![
                "list".to_string(),
                "table".to_string(),
                "inet".to_string(),
                "aegis_filter".to_string(),
            ],
        );
        let output = self.runner.run(req).await?;

        if output.is_success() {
            Ok(output.stdout)
        } else {
            Ok(String::new())
        }
    }

    /// Nạp Policy và trả về ApplyResult (Transaction an toàn)
    pub async fn apply_policy(&self, policy: &FirewallPolicy) -> Result<ApplyResult> {
        let compiler = NftablesCompiler::new();
        let compiled: CompiledFirewallPolicy = compiler.compile(policy)?;

        // 1. Kiểm tra cú pháp (Syntax Check) độc lập trên Candidate File
        self.check_syntax(&compiled.nft_script).await?;

        // 2. Tạo Snapshot trước khi áp dụng thay đổi
        let snapshot = self
            .create_snapshot(&format!(
                "Pre-apply snapshot for policy '{}'",
                policy.metadata.name
            ))
            .await?;

        // 3. Nạp kịch bản vào kernel
        self.apply_ruleset(&compiled.nft_script).await?;

        let execution_id = ExecutionId::new_v4();

        Ok(ApplyResult {
            execution_id,
            snapshot_id: snapshot.snapshot_id,
            applied: true,
            syntax_check_passed: true,
            validation_report: aegis_policy::PolicyValidator::validate(policy),
        })
    }

    /// Ghi script candidate ra đĩa tạm an toàn (atomic write)
    async fn write_candidate_file(&self, script: &str) -> Result<PathBuf> {
        if !self.candidate_file_dir.exists() {
            tokio::fs::create_dir_all(&self.candidate_file_dir)
                .await
                .map_err(|e| {
                    AegisError::Internal(format!("Failed to create candidate dir: {e}"))
                })?;
        }

        let file_name = format!("candidate_{}.nft", uuid::Uuid::new_v4());
        let file_path = self.candidate_file_dir.join(file_name);

        tokio::fs::write(&file_path, script)
            .await
            .map_err(|e| AegisError::Internal(format!("Failed to write candidate file: {e}")))?;

        Ok(file_path)
    }
}

#[async_trait]
impl FirewallBackend for NftablesRuntimeBackend {
    async fn check_syntax(&self, nft_script: &str) -> Result<()> {
        let candidate_path = self.write_candidate_file(nft_script).await?;

        let req = ProcessRequest::new(
            "nft",
            vec![
                "-c".to_string(),
                "-f".to_string(),
                candidate_path.to_string_lossy().to_string(),
            ],
        );

        let output = self.runner.run(req).await;

        // Dọn dẹp tệp candidate tạm thời sau khi kiểm tra cú pháp và ghi log nếu dọn dẹp thất bại
        if let Err(e) = tokio::fs::remove_file(&candidate_path).await {
            tracing::warn!("Could not remove candidate temp file '{candidate_path:?}': {e}");
        }

        let output = output?;
        if output.is_success() {
            Ok(())
        } else {
            Err(AegisError::Firewall(format!(
                "nftables syntax check failed: {}",
                output.stderr
            )))
        }
    }

    async fn apply_ruleset(&self, nft_script: &str) -> Result<()> {
        let candidate_path = self.write_candidate_file(nft_script).await?;

        let req = ProcessRequest::new(
            "nft",
            vec![
                "-f".to_string(),
                candidate_path.to_string_lossy().to_string(),
            ],
        );

        let output = self.runner.run(req).await;

        // Dọn dẹp tệp candidate tạm thời sau khi nạp ruleset và ghi log nếu dọn dẹp thất bại
        if let Err(e) = tokio::fs::remove_file(&candidate_path).await {
            tracing::warn!("Could not remove candidate temp file '{candidate_path:?}': {e}");
        }

        let output = output?;
        if output.is_success() {
            Ok(())
        } else {
            Err(AegisError::Firewall(format!(
                "Failed to apply nftables ruleset: {}",
                output.stderr
            )))
        }
    }

    async fn create_snapshot(&self, description: &str) -> Result<FirewallSnapshot> {
        let current_ruleset = self.read_kernel_ruleset().await?;
        self.snapshot_manager
            .create_snapshot("active", &current_ruleset, description)
            .await
    }

    async fn rollback_to_snapshot(&self, snapshot_id: &SnapshotId) -> Result<()> {
        let snapshot = self.snapshot_manager.read_snapshot(snapshot_id).await?;

        if snapshot.ruleset_content.trim().is_empty() {
            let req = ProcessRequest::new(
                "nft",
                vec![
                    "destroy".to_string(),
                    "table".to_string(),
                    "inet".to_string(),
                    "aegis_filter".to_string(),
                ],
            );
            let _ = self.runner.run(req).await;
            Ok(())
        } else {
            self.apply_ruleset(&snapshot.ruleset_content).await
        }
    }

    async fn inspect_state(&self) -> Result<FirewallState> {
        let managed_ruleset = self.read_managed_ruleset().await?;
        let rules_count = managed_ruleset
            .lines()
            .filter(|line| line.contains("counter") || line.contains("comment \"aegis:rule:"))
            .count();

        let managed_tables = if !managed_ruleset.is_empty() {
            vec!["inet aegis_filter".to_string()]
        } else {
            Vec::new()
        };

        Ok(FirewallState {
            managed_tables,
            rules_count,
            active_policy_hash: None,
        })
    }
}
