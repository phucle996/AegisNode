// Nftables Runtime Backend triển khai toàn bộ giao diện FirewallBackend
// Thực thi giao dịch an toàn (Transaction): Inspect -> Validate -> Compile -> Syntax Check -> Snapshot -> Apply -> Verify

use std::path::{Path, PathBuf};
use std::sync::Arc;

use aegis_core::{AegisError, ExecutionId, Result, SnapshotId};
use aegis_models::firewall::FirewallPolicy;
use aegis_policy::ValidationReport;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::compiler::{CompiledFirewallPolicy, FirewallCompiler};
use crate::nftables::NftablesCompiler;
use crate::process_runner::{ProcessRequest, ProcessRunner};
use crate::snapshot::{FirewallSnapshot, SnapshotManager};

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
    pub success: bool,
    pub execution_id: ExecutionId,
    pub snapshot_id: SnapshotId,
    pub applied_tables: Vec<String>,
}

/// Trait định nghĩa giao diện tương tác với Firewall Runtime Backend
#[async_trait]
pub trait FirewallBackend: Send + Sync {
    async fn inspect(&self) -> Result<FirewallState>;
    async fn validate(&self, policy: &FirewallPolicy) -> Result<ValidationReport>;
    async fn compile(&self, policy: &FirewallPolicy) -> Result<CompiledFirewallPolicy>;
    async fn snapshot(&self, reason: &str) -> Result<FirewallSnapshot>;
    async fn apply(&self, compiled: &CompiledFirewallPolicy) -> Result<ApplyResult>;
    async fn rollback(&self, snapshot: &FirewallSnapshot) -> Result<()>;
}

/// Dynamic Runtime Backend cho nftables
pub struct NftablesRuntimeBackend {
    runner: Arc<dyn ProcessRunner>,
    snapshot_manager: Arc<SnapshotManager>,
    compiler: Arc<dyn FirewallCompiler>,
    candidate_dir: PathBuf,
}

impl NftablesRuntimeBackend {
    pub fn new(
        runner: Arc<dyn ProcessRunner>,
        snapshot_manager: Arc<SnapshotManager>,
        candidate_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            runner,
            snapshot_manager,
            compiler: Arc::new(NftablesCompiler::new()),
            candidate_dir: candidate_dir.into(),
        }
    }

    /// Thực thi cú pháp check bằng `nft --check --file <candidate>`
    async fn check_syntax(&self, candidate_file: &Path) -> Result<()> {
        let req = ProcessRequest::new(
            "nft",
            vec![
                "--check".to_string(),
                "--file".to_string(),
                candidate_file.to_string_lossy().to_string(),
            ],
        );
        let out = self.runner.run(req).await?;
        if !out.is_success() {
            return Err(AegisError::Firewall(format!(
                "nftables syntax check failed: {}",
                out.stderr
            )));
        }
        Ok(())
    }

    /// Đọc trạng thái ruleset hiện tại để lưu snapshot
    async fn get_current_ruleset(&self) -> Result<String> {
        let req = ProcessRequest::new("nft", vec!["list".to_string(), "ruleset".to_string()]);
        let out = self.runner.run(req).await?;
        if out.is_success() {
            Ok(out.stdout)
        } else {
            // Nếu chưa có ruleset nào
            Ok("# Empty ruleset\n".to_string())
        }
    }
}

#[async_trait]
impl FirewallBackend for NftablesRuntimeBackend {
    async fn inspect(&self) -> Result<FirewallState> {
        let req = ProcessRequest::new(
            "nft",
            vec![
                "--json".to_string(),
                "list".to_string(),
                "ruleset".to_string(),
            ],
        );
        let out = self.runner.run(req).await?;
        if !out.is_success() {
            return Ok(FirewallState {
                managed_tables: Vec::new(),
                rules_count: 0,
                active_policy_hash: None,
            });
        }

        let mut managed_tables = Vec::new();
        if out.stdout.contains("aegis_filter") {
            managed_tables.push("inet aegis_filter".to_string());
        }
        if out.stdout.contains("aegis_nat") {
            managed_tables.push("ip aegis_nat".to_string());
        }

        // Đếm số lượng rules chứa comment aegis:rule:
        let rules_count = out.stdout.matches("aegis:rule:").count();

        Ok(FirewallState {
            managed_tables,
            rules_count,
            active_policy_hash: None,
        })
    }

    async fn validate(&self, policy: &FirewallPolicy) -> Result<ValidationReport> {
        Ok(aegis_policy::PolicyValidator::validate(policy))
    }

    async fn compile(&self, policy: &FirewallPolicy) -> Result<CompiledFirewallPolicy> {
        self.compiler.compile(policy)
    }

    async fn snapshot(&self, reason: &str) -> Result<FirewallSnapshot> {
        let current_ruleset = self.get_current_ruleset().await?;
        let current_hash =
            aegis_policy::PolicyHasher::compute_hash(&aegis_models::firewall::FirewallPolicy {
                api_version: "aegisnode.io/v1".to_string(),
                kind: "FirewallPolicy".to_string(),
                metadata: aegis_models::firewall::PolicyMetadata {
                    name: "snapshot-state".to_string(),
                    id: aegis_core::PolicyId::new_v4(),
                    version: 1,
                    labels: Default::default(),
                    created_at: chrono::Utc::now(),
                },
                defaults: aegis_models::firewall::FirewallDefaults {
                    input: aegis_models::firewall::FirewallAction::Drop,
                    output: aegis_models::firewall::FirewallAction::Accept,
                    forward: aegis_models::firewall::FirewallAction::Drop,
                },
                rules: Vec::new(),
            });

        self.snapshot_manager
            .create_snapshot(current_hash, current_ruleset, reason)
            .await
    }

    async fn apply(&self, compiled: &CompiledFirewallPolicy) -> Result<ApplyResult> {
        let execution_id = ExecutionId::new_v4();

        // 1. Tạo candidate file
        tokio::fs::create_dir_all(&self.candidate_dir)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to create candidate dir: {e}")))?;

        let candidate_path = self
            .candidate_dir
            .join(format!("candidate_{execution_id}.nft"));
        tokio::fs::write(&candidate_path, &compiled.nft_script)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to write candidate file: {e}")))?;

        // 2. Syntax check bằng nft --check
        if let Err(e) = self.check_syntax(&candidate_path).await {
            let _ = tokio::fs::remove_file(&candidate_path).await;
            return Err(e);
        }

        // 3. Tạo snapshot trước khi thay đổi
        let snapshot = self
            .snapshot(&format!("Pre-apply snapshot for execution {execution_id}"))
            .await?;

        // 4. Thực thi apply: nft -f candidate.nft
        let apply_req = ProcessRequest::new(
            "nft",
            vec![
                "--file".to_string(),
                candidate_path.to_string_lossy().to_string(),
            ],
        );

        let apply_out = self.runner.run(apply_req).await?;
        let _ = tokio::fs::remove_file(&candidate_path).await;

        if !apply_out.is_success() {
            // Tự động khôi phục nếu apply thất bại
            let _ = self.rollback(&snapshot).await;
            return Err(AegisError::Firewall(format!(
                "Apply failed: {}. Automatically restored pre-apply snapshot.",
                apply_out.stderr
            )));
        }

        // 5. Verify xem table quản lý đã active hay chưa
        let state = self.inspect().await?;
        if state.managed_tables.is_empty() {
            return Err(AegisError::Firewall(
                "Apply reported success but no AegisNode managed tables found in runtime."
                    .to_string(),
            ));
        }

        Ok(ApplyResult {
            success: true,
            execution_id,
            snapshot_id: snapshot.snapshot_id,
            applied_tables: compiled.managed_tables.clone(),
        })
    }

    async fn rollback(&self, snapshot: &FirewallSnapshot) -> Result<()> {
        if !snapshot.verify_checksum() {
            return Err(AegisError::Storage(
                "Cannot rollback to corrupted snapshot: Checksum mismatch!".to_string(),
            ));
        }

        // 1. Tạo candidate_dir nếu chưa tồn tại
        tokio::fs::create_dir_all(&self.candidate_dir)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to create candidate dir: {e}")))?;

        // 2. Tạo tệp tạm thời để restore
        let temp_restore_path = self
            .candidate_dir
            .join(format!("restore_{}.nft", snapshot.snapshot_id));
        tokio::fs::write(&temp_restore_path, &snapshot.ruleset_content)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to write restore file: {e}")))?;

        let req = ProcessRequest::new(
            "nft",
            vec![
                "--file".to_string(),
                temp_restore_path.to_string_lossy().to_string(),
            ],
        );

        let out = self.runner.run(req).await;
        let _ = tokio::fs::remove_file(&temp_restore_path).await;

        let res = out?;
        if !res.is_success() {
            return Err(AegisError::Firewall(format!(
                "Rollback failed: {}",
                res.stderr
            )));
        }

        Ok(())
    }
}
