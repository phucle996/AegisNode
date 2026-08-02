// Integration tests cho NftablesRuntimeBackend và SnapshotManager trong aegis-firewall

use std::sync::Arc;

use aegis_firewall::{
    FirewallBackend, MockProcessRunner, NftablesRuntimeBackend, ProcessOutput, SnapshotManager,
};
use aegis_models::firewall::FirewallPolicy;

#[tokio::test]
async fn test_snapshot_manager_create_and_read() {
    let temp_dir = std::env::temp_dir().join(format!("aegis_snap_test_{}", uuid::Uuid::new_v4()));
    let manager = SnapshotManager::new(&temp_dir, 5);

    let content = "table inet aegis_filter { chain input { policy drop; } }";
    let snapshot = manager
        .create_snapshot("policy-hash-123", content, "Test snapshot")
        .await
        .expect("Failed to create snapshot");

    assert!(snapshot.verify_checksum());
    assert_eq!(snapshot.policy_hash, "policy-hash-123");

    // Đọc lại từ disk
    let read_back = manager
        .read_snapshot(&snapshot.snapshot_id)
        .await
        .expect("Failed to read snapshot");

    assert_eq!(read_back.snapshot_id, snapshot.snapshot_id);
    assert_eq!(read_back.ruleset_content, content);
    assert!(read_back.verify_checksum());

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_backend_apply_transaction_with_mock() {
    let runner = Arc::new(MockProcessRunner::new());

    // Đăng ký mock response cho inspect và syntax check
    runner.register_response(
        "nft --json list ruleset",
        ProcessOutput::success(r#"{"nftables":[{"table":{"family":"inet","name":"aegis_filter"}},{"rule":{"comment":"aegis:rule:r1"}}]}"#),
    );

    let temp_snap_dir = std::env::temp_dir().join(format!("aegis_snap_{}", uuid::Uuid::new_v4()));
    let temp_cand_dir = std::env::temp_dir().join(format!("aegis_cand_{}", uuid::Uuid::new_v4()));

    let snapshot_manager = Arc::new(SnapshotManager::new(&temp_snap_dir, 5));
    let backend = NftablesRuntimeBackend::new(runner, snapshot_manager, &temp_cand_dir);

    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let compiled = backend.compile(&policy).await.expect("Failed to compile");
    let apply_res = backend.apply(&compiled).await.expect("Apply failed");

    assert!(apply_res.success);
    assert_eq!(apply_res.applied_tables, vec!["inet aegis_filter"]);

    let _ = tokio::fs::remove_dir_all(&temp_snap_dir).await;
    let _ = tokio::fs::remove_dir_all(&temp_cand_dir).await;
}

#[tokio::test]
async fn test_backend_rollback_with_mock() {
    let runner = Arc::new(MockProcessRunner::new());
    let temp_snap_dir =
        std::env::temp_dir().join(format!("aegis_snap_rb_{}", uuid::Uuid::new_v4()));
    let temp_cand_dir =
        std::env::temp_dir().join(format!("aegis_cand_rb_{}", uuid::Uuid::new_v4()));

    let snapshot_manager = Arc::new(SnapshotManager::new(&temp_snap_dir, 5));
    let backend = NftablesRuntimeBackend::new(runner, snapshot_manager.clone(), &temp_cand_dir);

    let snapshot = snapshot_manager
        .create_snapshot("old-hash", "table inet aegis_filter {}", "Pre-change")
        .await
        .unwrap();

    let rollback_res = backend.rollback(&snapshot).await;
    assert!(rollback_res.is_ok());

    let _ = tokio::fs::remove_dir_all(&temp_snap_dir).await;
    let _ = tokio::fs::remove_dir_all(&temp_cand_dir).await;
}
