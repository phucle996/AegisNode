// Integration tests cho SafeApplyManager, Rollback Timer và Concurrent Apply Lock trong aegis-firewall

use std::sync::Arc;
use std::time::Duration;

use aegis_firewall::{
    ExecutionState, MockProcessRunner, NftablesRuntimeBackend, ProcessOutput, SafeApplyManager,
    SnapshotManager,
};
use aegis_models::firewall::FirewallPolicy;

fn setup_test_manager() -> (SafeApplyManager, PathBufCleaner) {
    let runner = Arc::new(MockProcessRunner::new());

    runner.register_response(
        "nft --json list ruleset",
        ProcessOutput::success(
            r#"{"nftables":[{"table":{"family":"inet","name":"aegis_filter"}}]}"#,
        ),
    );
    runner.register_response(
        "nft list tables",
        ProcessOutput::success("table inet aegis_filter"),
    );
    runner.register_response(
        "ip link show lo",
        ProcessOutput::success("1: lo: <LOOPBACK,UP,LOWER_UP>"),
    );
    runner.register_response(
        "ping -c 1 127.0.0.1",
        ProcessOutput::success("1 packets transmitted, 1 received, 0% packet loss"),
    );

    let temp_snap_dir =
        std::env::temp_dir().join(format!("aegis_safe_snap_{}", uuid::Uuid::new_v4()));
    let temp_cand_dir =
        std::env::temp_dir().join(format!("aegis_safe_cand_{}", uuid::Uuid::new_v4()));

    let snapshot_manager = Arc::new(SnapshotManager::new(&temp_snap_dir, 5));
    let backend = Arc::new(NftablesRuntimeBackend::new(
        runner.clone(),
        snapshot_manager,
        &temp_cand_dir,
    ));

    let manager = SafeApplyManager::new(backend, runner);
    let cleaner = PathBufCleaner {
        paths: vec![temp_snap_dir, temp_cand_dir],
    };

    (manager, cleaner)
}

struct PathBufCleaner {
    paths: Vec<std::path::PathBuf>,
}

impl Drop for PathBufCleaner {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

#[tokio::test]
async fn test_safe_apply_confirm_flow() {
    let (manager, _cleaner) = setup_test_manager();
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let exec = manager
        .execute_safe_apply(&policy, 10)
        .await
        .expect("Safe apply failed");

    assert_eq!(exec.state, ExecutionState::AppliedPendingConfirmation);

    let confirmed = manager
        .confirm(&exec.execution_id)
        .await
        .expect("Confirm failed");

    assert_eq!(confirmed.state, ExecutionState::Committed);
}

#[tokio::test]
async fn test_safe_apply_automatic_rollback_on_timeout() {
    let (manager, _cleaner) = setup_test_manager();
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    // Cấu hình rollback timer ngắn: 1 giây
    let exec = manager
        .execute_safe_apply(&policy, 1)
        .await
        .expect("Safe apply failed");

    assert_eq!(exec.state, ExecutionState::AppliedPendingConfirmation);

    // Chờ 1.2 giây để Rollback Timer tự động kích hoạt
    tokio::time::sleep(Duration::from_millis(1200)).await;

    let updated = manager
        .get_execution(&exec.execution_id)
        .expect("Execution missing");

    assert_eq!(updated.state, ExecutionState::RolledBack);
    assert!(updated.error_message.unwrap().contains("timeout"));
}

#[tokio::test]
async fn test_safe_apply_manual_rollback() {
    let (manager, _cleaner) = setup_test_manager();
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let exec = manager
        .execute_safe_apply(&policy, 30)
        .await
        .expect("Safe apply failed");

    assert_eq!(exec.state, ExecutionState::AppliedPendingConfirmation);

    let rolled_back = manager
        .rollback(&exec.execution_id)
        .await
        .expect("Rollback failed");

    assert_eq!(rolled_back.state, ExecutionState::RolledBack);
}

#[tokio::test]
async fn test_safe_apply_concurrent_lock() {
    let (manager, _cleaner) = setup_test_manager();
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let exec1 = manager
        .execute_safe_apply(&policy, 30)
        .await
        .expect("Safe apply 1 failed");

    assert_eq!(exec1.state, ExecutionState::AppliedPendingConfirmation);

    // Thử nghiệm apply lần 2 khi lần 1 chưa confirm -> Phải bị từ chối với AegisError::Conflict
    let res2 = manager.execute_safe_apply(&policy, 30).await;
    assert!(res2.is_err());
    let err_msg = format!("{:?}", res2.err());
    assert!(err_msg.contains("Conflict") || err_msg.contains("in progress"));

    // Confirm đợt 1 để giải phóng lock
    let _ = manager.confirm(&exec1.execution_id).await;

    // Sau khi confirm, apply mới có thể chạy thành công
    let exec2 = manager.execute_safe_apply(&policy, 30).await;
    assert!(exec2.is_ok());
}
