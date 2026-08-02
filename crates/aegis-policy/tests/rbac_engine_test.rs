// Integration Test cho RBAC Engine & Anti Self-Approval Workflow (Phase 21)

use aegis_models::rbac::{AccessScope, ApprovalRecord, Permission, RiskLevel, Role, UserSubject};
use aegis_policy::{ApprovalWorkflowValidator, RbacEngine};

#[test]
fn test_rbac_engine_permission_evaluation() {
    // 1. Khởi tạo User Viewer
    let viewer = UserSubject {
        id: "user-01".to_string(),
        username: "alice".to_string(),
        roles: vec![Role::Viewer],
    };

    // Viewer có quyền FirewallRead
    assert!(RbacEngine::authorize(&viewer, Permission::FirewallRead, &AccessScope::Global).is_ok());
    // Viewer KHÔNG có quyền FirewallApply
    assert!(
        RbacEngine::authorize(&viewer, Permission::FirewallApply, &AccessScope::Global).is_err()
    );

    // 2. Khởi tạo User SecurityAdmin
    let sec_admin = UserSubject {
        id: "user-02".to_string(),
        username: "bob".to_string(),
        roles: vec![Role::SecurityAdmin],
    };

    // SecurityAdmin có quyền FirewallApply & ChangePlanApprove
    assert!(
        RbacEngine::authorize(&sec_admin, Permission::FirewallApply, &AccessScope::Global).is_ok()
    );
    assert!(
        RbacEngine::authorize(
            &sec_admin,
            Permission::ChangePlanApprove,
            &AccessScope::Global
        )
        .is_ok()
    );
}

#[test]
fn test_approval_workflow_anti_self_approval() {
    let creator_id = "user-creator-01";

    // 1. Chữ ký phê duyệt bởi chính người tạo -> Phải bị từ chối
    let self_approval = vec![ApprovalRecord {
        approver_id: "user-creator-01".to_string(),
        approver_username: "creator_alice".to_string(),
        approved_at: "2026-08-02T12:00:00Z".to_string(),
        risk_level: RiskLevel::Medium,
        comments: None,
    }];

    let result = ApprovalWorkflowValidator::validate_approval_chain(
        creator_id,
        &self_approval,
        RiskLevel::Medium,
    );
    assert!(
        result.is_err(),
        "Tự phê duyệt plan của chính mình phải bị từ chối với Anti Self-Approval error"
    );
}

#[test]
fn test_approval_workflow_critical_two_person_approval() {
    let creator_id = "user-creator-01";

    // 1. Chỉ có 1 chữ ký cho Critical Risk Plan -> Thất bại
    let single_approval = vec![ApprovalRecord {
        approver_id: "user-approver-01".to_string(),
        approver_username: "bob".to_string(),
        approved_at: "2026-08-02T12:00:00Z".to_string(),
        risk_level: RiskLevel::Critical,
        comments: None,
    }];

    assert!(
        ApprovalWorkflowValidator::validate_approval_chain(
            creator_id,
            &single_approval,
            RiskLevel::Critical
        )
        .is_err(),
        "Critical plan chỉ có 1 chữ ký phải bị từ chối"
    );

    // 2. Có đủ 2 chữ ký từ 2 người duyệt khác nhau -> Thành công
    let dual_approvals = vec![
        ApprovalRecord {
            approver_id: "user-approver-01".to_string(),
            approver_username: "bob".to_string(),
            approved_at: "2026-08-02T12:00:00Z".to_string(),
            risk_level: RiskLevel::Critical,
            comments: None,
        },
        ApprovalRecord {
            approver_id: "user-approver-02".to_string(),
            approver_username: "charlie".to_string(),
            approved_at: "2026-08-02T12:01:00Z".to_string(),
            risk_level: RiskLevel::Critical,
            comments: None,
        },
    ];

    assert!(
        ApprovalWorkflowValidator::validate_approval_chain(
            creator_id,
            &dual_approvals,
            RiskLevel::Critical
        )
        .is_ok(),
        "Critical plan có đủ 2 chữ ký từ 2 người duyệt khác nhau phải được thông qua"
    );
}
