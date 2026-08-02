// Integration Test cho Cryptographic Audit Hash Chain (Phase 25 Audit & Recovery)

use aegis_audit::{AuditChainRecord, AuditChainVerifier, GENESIS_AUDIT_HASH};

fn create_sample_chain() -> Vec<AuditChainRecord> {
    // Record 1
    let mut r1 = AuditChainRecord {
        id: "audit-001".to_string(),
        actor_id: "user-admin".to_string(),
        action: "POLICY_APPLY".to_string(),
        resource: "policy:web-prod".to_string(),
        node_id: Some("node-01".to_string()),
        execution_id: Some("exec-01".to_string()),
        before_hash: None,
        after_hash: Some("hash-v1".to_string()),
        result: "SUCCESS".to_string(),
        timestamp: "2026-08-02T12:00:00Z".to_string(),
        sequence_number: 1,
        prev_event_hash: GENESIS_AUDIT_HASH.to_string(),
        event_hash: "".to_string(),
    };
    r1.event_hash = r1.compute_event_hash();

    // Record 2
    let mut r2 = AuditChainRecord {
        id: "audit-002".to_string(),
        actor_id: "user-sec-admin".to_string(),
        action: "RBAC_APPROVE".to_string(),
        resource: "change_plan:plan-01".to_string(),
        node_id: None,
        execution_id: None,
        before_hash: None,
        after_hash: Some("hash-v2".to_string()),
        result: "SUCCESS".to_string(),
        timestamp: "2026-08-02T12:05:00Z".to_string(),
        sequence_number: 2,
        prev_event_hash: r1.event_hash.clone(),
        event_hash: "".to_string(),
    };
    r2.event_hash = r2.compute_event_hash();

    vec![r1, r2]
}

#[test]
fn test_valid_audit_chain_verification() {
    let chain = create_sample_chain();
    let result = AuditChainVerifier::verify_chain_integrity(&chain);

    assert!(
        result.is_ok(),
        "Chuỗi Audit Hash Chain hợp lệ phải xác thực thành công"
    );
}

#[test]
fn test_tampered_audit_record_rejection() {
    let mut chain = create_sample_chain();

    // Sửa đổi 1 byte trong thông tin actor_id của bản ghi r1 (Tấn công làm sai lệch lịch sử)
    chain[0].actor_id = "attacker-hacker".to_string();

    let result = AuditChainVerifier::verify_chain_integrity(&chain);
    assert!(
        result.is_err(),
        "Sửa đổi dữ liệu trong Audit log phải bị từ chối do event_hash không khớp"
    );
}

#[test]
fn test_disrupted_merkle_link_rejection() {
    let mut chain = create_sample_chain();

    // Làm gián đoạn Merkle link prev_event_hash của bản ghi r2
    chain[1].prev_event_hash =
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

    let result = AuditChainVerifier::verify_chain_integrity(&chain);
    assert!(
        result.is_err(),
        "Làm gián đoạn prev_event_hash Merkle link phải bị từ chối do gián đoạn chuỗi Audit"
    );
}
