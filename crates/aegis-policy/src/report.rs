// Model báo cáo kết quả kiểm tra Policy Validation (Validation Report & Issues)
// Phân loại nghiêm ngặt giữa ERROR (chặn apply), WARNING (cảnh báo bảo mật) và INFO

use aegis_core::RuleId;
use serde::{Deserialize, Serialize};

/// Mức độ nghiêm trọng của vấn đề phát hiện trong Policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Một vấn đề được phát hiện trong quá trình kiểm tra Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<RuleId>,
}

impl ValidationIssue {
    /// Tạo một vấn đề loại ERROR (chặn apply)
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        rule_id: Option<RuleId>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            code: code.into(),
            message: message.into(),
            field_path,
            rule_id,
        }
    }

    /// Tạo một vấn đề loại WARNING (cảnh báo bảo mật)
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        rule_id: Option<RuleId>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            code: code.into(),
            message: message.into(),
            field_path,
            rule_id,
        }
    }

    /// Tạo một vấn đề loại INFO (thông tin bổ sung)
    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        rule_id: Option<RuleId>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Info,
            code: code.into(),
            message: message.into(),
            field_path,
            rule_id,
        }
    }
}

/// Báo cáo kiểm tra tổng thể của Firewall Policy
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub infos: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Thêm một ValidationIssue vào báo cáo và cập nhật trạng thái `valid`
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            ValidationSeverity::Error => {
                self.valid = false;
                self.errors.push(issue);
            }
            ValidationSeverity::Warning => {
                self.warnings.push(issue);
            }
            ValidationSeverity::Info => {
                self.infos.push(issue);
            }
        }
    }

    /// Kiểm tra Policy có hoàn toàn hợp lệ (không chứa ERROR) hay không
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }
}
