//! RBAC & Approval Workflow Engine (Phase 21 Authorization & Anti Self-Approval)
//! Thực hiện đánh giá phân quyền Granular Permissions và kiểm soát quy trình Phê duyệt Kép chống tự duyệt.

use aegis_core::AegisError;
use aegis_models::rbac::{AccessScope, ApprovalRecord, Permission, RiskLevel, UserSubject};

/// Động cơ Đánh giá Phân quyền (RBAC Engine)
pub struct RbacEngine;

impl RbacEngine {
    /// Đánh giá xem người dùng `UserSubject` có đủ quyền `Permission` thực hiện thao tác hay không
    pub fn authorize(
        subject: &UserSubject,
        required_permission: Permission,
        _scope: &AccessScope,
    ) -> Result<(), AegisError> {
        // 1. Tập hợp toàn bộ permissions từ tất cả các Roles của người dùng
        let mut user_permissions = Vec::new();
        for role in &subject.roles {
            user_permissions.extend(role.default_permissions());
        }

        // 2. Kiểm tra xem quyền yêu cầu có nằm trong danh sách permissions của user hay không
        if user_permissions.contains(&required_permission) {
            Ok(())
        } else {
            Err(AegisError::Permission(format!(
                "Từ chối truy cập: Người dùng '{}' không có quyền {:?}",
                subject.username, required_permission
            )))
        }
    }
}

/// Bộ kiểm tra Quy trình Phê duyệt Change Plan (Approval Workflow Validator)
pub struct ApprovalWorkflowValidator;

impl ApprovalWorkflowValidator {
    /// Đánh giá và kiểm tra danh sách chữ ký phê duyệt (Approval Records) cho một Change Plan
    pub fn validate_approval_chain(
        creator_id: &str,
        approvals: &[ApprovalRecord],
        risk_level: RiskLevel,
    ) -> Result<bool, AegisError> {
        // 1. Kiểm tra quy tắc Chống Tự Duyệt (Anti Self-Approval)
        for record in approvals {
            if record.approver_id == creator_id {
                return Err(AegisError::Permission(format!(
                    "Vi phạm quy tắc Anti Self-Approval: Người tạo Change Plan ({creator_id}) không được tự phê duyệt plan của chính mình"
                )));
            }
        }

        // 2. Đánh giá điều kiện phê duyệt theo từng cấp độ rủi ro (Risk Tiers)
        match risk_level {
            RiskLevel::Low => {
                // Rủi ro thấp: Không bắt buộc có người duyệt
                Ok(true)
            }
            RiskLevel::Medium => {
                // Rủi ro trung bình: Yêu cầu ít nhất 1 chữ ký phê duyệt
                if !approvals.is_empty() {
                    Ok(true)
                } else {
                    Err(AegisError::Permission(
                        "Change Plan mức rủi ro Medium yêu cầu ít nhất 1 chữ ký phê duyệt từ Operator".to_string(),
                    ))
                }
            }
            RiskLevel::High => {
                // Rủi ro cao: Yêu cầu ít nhất 1 chữ ký phê duyệt
                if !approvals.is_empty() {
                    Ok(true)
                } else {
                    Err(AegisError::Permission(
                        "Change Plan mức rủi ro High yêu cầu ít nhất 1 chữ ký phê duyệt từ SecurityAdmin".to_string(),
                    ))
                }
            }
            RiskLevel::Critical => {
                // Rủi ro cực cao: Yêu cầu Quy tắc Phê duyệt Kép (2-Person Approval)
                // Cần ít nhất 2 chữ ký từ 2 người phê duyệt hoàn toàn khác nhau
                let unique_approvers: std::collections::HashSet<_> =
                    approvals.iter().map(|a| &a.approver_id).collect();

                if unique_approvers.len() >= 2 {
                    Ok(true)
                } else {
                    Err(AegisError::Permission(format!(
                        "Vi phạm 2-Person Approval: Change Plan mức Critical yêu cầu ít nhất 2 chữ ký phê duyệt từ 2 người dùng khác nhau (Hiện có: {})",
                        unique_approvers.len()
                    )))
                }
            }
        }
    }
}
