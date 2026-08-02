// Integration Test cho Automated Backup & Disaster Recovery Engine (Phase 25 Audit & Recovery)

use aegis_storage::{BackupEngine, CURRENT_BACKUP_SCHEMA_VERSION};

#[test]
fn test_valid_backup_creation_and_verification() {
    let policies = r#"[{"id": "p1", "name": "web-policy"}]"#.to_string();
    let nodes = r#"[{"id": "n1", "hostname": "worker-01"}]"#.to_string();
    let audit_logs = r#"[{"id": "a1", "action": "APPLY"}]"#.to_string();
    let created_at = "2026-08-02T12:00:00Z".to_string();

    // 1. Tạo bản sao lưu BackupSnapshot
    let snapshot = BackupEngine::create_backup(policies, nodes, audit_logs, created_at);

    assert_eq!(snapshot.version, CURRENT_BACKUP_SCHEMA_VERSION);
    assert!(!snapshot.checksum.is_empty());

    // 2. Verification phải thành công với bản sao lưu hợp lệ
    let result = BackupEngine::verify_backup(&snapshot);
    assert!(
        result.is_ok(),
        "Xác thực bản sao lưu BackupSnapshot hợp lệ phải thành công"
    );
}

#[test]
fn test_corrupted_backup_checksum_rejection() {
    let policies = r#"[{"id": "p1", "name": "web-policy"}]"#.to_string();
    let nodes = r#"[{"id": "n1", "hostname": "worker-01"}]"#.to_string();
    let audit_logs = r#"[{"id": "a1", "action": "APPLY"}]"#.to_string();

    let mut snapshot = BackupEngine::create_backup(
        policies,
        nodes,
        audit_logs,
        "2026-08-02T12:00:00Z".to_string(),
    );

    // Sửa đổi 1 byte trong dữ liệu policies_json của file backup
    snapshot.policies_json = r#"[{"id": "p1", "name": "HACKED-POLICY"}]"#.to_string();

    // Verification phải báo lỗi Checksum mismatch
    let result = BackupEngine::verify_backup(&snapshot);
    assert!(
        result.is_err(),
        "File backup bị hư hỏng hoặc sửa đổi checksum phải bị từ chối trước khi restore"
    );
}
