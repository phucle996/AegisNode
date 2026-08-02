// Integration tests cho AegisNode Storage Crate và SQLite Repositories

use aegis_core::{ExecutionId, PolicyId, SnapshotId};
use aegis_models::firewall::FirewallPolicy;
use aegis_storage::{
    AuditRepository, ExecutionRepository, PolicyRepository, SqliteRepository, init_in_memory_pool,
};

#[tokio::test]
async fn test_sqlite_repository_policy_and_audit() {
    let pool = init_in_memory_pool()
        .await
        .expect("Failed to init in-memory sqlite");
    let repo = SqliteRepository::new(pool);

    let yaml_str = include_str!("../../../tests/fixtures/policies/minimal.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    // 1. Test save_policy & get_latest_policy
    repo.save_policy(&policy, "hash-123456")
        .await
        .expect("Failed to save policy");

    let loaded = repo
        .get_latest_policy()
        .await
        .expect("Failed to get latest policy");

    assert!(loaded.is_some());
    let loaded_policy = loaded.unwrap();
    assert_eq!(loaded_policy.metadata.name, policy.metadata.name);

    // 2. Test ExecutionRepository
    let exec_id = ExecutionId::new_v4();
    let pol_id = PolicyId::new_v4();
    let snap_id = SnapshotId::new_v4();

    repo.save_execution(
        &exec_id,
        &pol_id,
        &snap_id,
        "APPLIED_PENDING_CONFIRMATION",
        30,
    )
    .await
    .expect("Failed to save execution");

    repo.update_execution_state(&exec_id, "COMMITTED", None)
        .await
        .expect("Failed to update execution");

    // 3. Test AuditRepository
    repo.record_audit("TEST_ACTION", "admin", "res-1", r#"{"key":"val"}"#)
        .await
        .expect("Failed to record audit");

    let logs = repo.list_audits(10).await.expect("Failed to list audits");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "TEST_ACTION");
    assert_eq!(logs[0].actor, "admin");
}
