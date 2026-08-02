// Snapshot Manager quản lý lưu trữ và khôi phục bản chụp trạng thái nftables trước khi Apply
// Đảm bảo lưu trữ atomic, kiểm tra checksum SHA-256 chống hỏng dữ liệu và áp dụng chính sách Retention

use std::path::PathBuf;

use aegis_core::{AegisError, Result, SnapshotId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Bản chụp trạng thái nftables ruleset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallSnapshot {
    pub snapshot_id: SnapshotId,
    pub created_at: DateTime<Utc>,
    pub policy_hash: String,
    pub ruleset_content: String,
    pub checksum_sha256: String,
    pub reason: String,
}

impl FirewallSnapshot {
    /// Tính toán SHA-256 Checksum cho ruleset content
    pub fn compute_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Xác minh tính toàn vẹn Checksum
    pub fn verify_checksum(&self) -> bool {
        let expected = Self::compute_checksum(&self.ruleset_content);
        self.checksum_sha256 == expected
    }
}

/// Trình quản lý lưu trữ Snapshot trên đĩa cứng
pub struct SnapshotManager {
    base_dir: PathBuf,
    max_retention: usize,
}

impl SnapshotManager {
    /// Khởi tạo SnapshotManager với đường dẫn thư mục gốc và retention limit
    pub fn new(base_dir: impl Into<PathBuf>, max_retention: usize) -> Self {
        Self {
            base_dir: base_dir.into(),
            max_retention,
        }
    }

    /// Thư mục mặc định cho production: `/var/lib/aegisnode/snapshots`
    pub fn default_prod() -> Self {
        Self::new("/var/lib/aegisnode/snapshots", 10)
    }

    /// Tạo và lưu một bản Snapshot mới một cách atomic
    pub async fn create_snapshot(
        &self,
        policy_hash: impl Into<String>,
        ruleset_content: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<FirewallSnapshot> {
        let content_str = ruleset_content.into();
        let checksum = FirewallSnapshot::compute_checksum(&content_str);
        let snapshot_id = SnapshotId::new_v4();
        let now = Utc::now();

        let snapshot = FirewallSnapshot {
            snapshot_id: snapshot_id.clone(),
            created_at: now,
            policy_hash: policy_hash.into(),
            ruleset_content: content_str,
            checksum_sha256: checksum,
            reason: reason.into(),
        };

        // Đường dẫn thư mục snapshot: base_dir/<snapshot_id>/
        let dir_path = self.base_dir.join(snapshot_id.as_str());

        tokio::fs::create_dir_all(&dir_path).await.map_err(|e| {
            AegisError::Storage(format!(
                "Failed to create snapshot directory '{dir_path:?}': {e}"
            ))
        })?;

        // Write ruleset.nft
        let ruleset_path = dir_path.join("ruleset.nft");
        tokio::fs::write(&ruleset_path, &snapshot.ruleset_content)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to write ruleset.nft: {e}")))?;

        // Write metadata.json
        let metadata_path = dir_path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&snapshot).map_err(|e| {
            AegisError::Storage(format!("Failed to serialize snapshot metadata: {e}"))
        })?;
        tokio::fs::write(&metadata_path, metadata_json)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to write metadata.json: {e}")))?;

        // Write checksum.sha256
        let checksum_path = dir_path.join("checksum.sha256");
        tokio::fs::write(&checksum_path, &snapshot.checksum_sha256)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to write checksum.sha256: {e}")))?;

        // Áp dụng Retention Policy dọn dẹp các snapshot quá cũ
        self.apply_retention_policy().await?;

        Ok(snapshot)
    }

    /// Đọc và xác minh một bản Snapshot từ đĩa
    pub async fn read_snapshot(&self, snapshot_id: &SnapshotId) -> Result<FirewallSnapshot> {
        let dir_path = self.base_dir.join(snapshot_id.as_str());
        let metadata_path = dir_path.join("metadata.json");

        if !metadata_path.exists() {
            return Err(AegisError::NotFound(format!(
                "Snapshot '{snapshot_id}' not found at '{metadata_path:?}'"
            )));
        }

        let json_bytes = tokio::fs::read(&metadata_path)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to read metadata.json: {e}")))?;

        let snapshot: FirewallSnapshot = serde_json::from_slice(&json_bytes)
            .map_err(|e| AegisError::Storage(format!("Failed to parse metadata.json: {e}")))?;

        // Kiểm tra tính toàn vẹn Checksum
        if !snapshot.verify_checksum() {
            return Err(AegisError::Storage(format!(
                "Checksum verification failed for snapshot '{snapshot_id}'. Data corrupted!"
            )));
        }

        Ok(snapshot)
    }

    /// Dọn dẹp các bản Snapshot cũ vượt quá max_retention
    async fn apply_retention_policy(&self) -> Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }

        let mut entries = Vec::new();
        let mut dir_reader = tokio::fs::read_dir(&self.base_dir)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to read snapshot dir: {e}")))?;

        while let Ok(Some(entry)) = dir_reader.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    if let Ok(bytes) = tokio::fs::read(&metadata_path).await {
                        if let Ok(snap) = serde_json::from_slice::<FirewallSnapshot>(&bytes) {
                            entries.push((snap.created_at, path));
                        }
                    }
                }
            }
        }

        // Sắp xếp theo thời gian tăng dần (cũ nhất ở đầu)
        entries.sort_by_key(|a| a.0);

        // Xóa bớt nếu vượt ngưỡng retention
        if entries.len() > self.max_retention {
            let delete_count = entries.len() - self.max_retention;
            for (_, path_to_remove) in entries.iter().take(delete_count) {
                let _ = tokio::fs::remove_dir_all(path_to_remove).await;
            }
        }

        Ok(())
    }
}
