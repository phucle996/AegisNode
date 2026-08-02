// Engine kiểm tra tính hợp lệ về cú pháp (Schema) và logic bảo mật (Semantic) của Firewall Policy
// Ngăn ngừa triệt để việc apply các cấu hình lỗi gây mất kết nối node hoặc hổng lỗ hổng an ninh

use std::collections::HashSet;

use aegis_models::firewall::{
    ConnectionState, FirewallAction, FirewallDirection, FirewallPolicy, InterfaceSelector,
    PortSpec, TransportProtocol,
};

use crate::report::{ValidationIssue, ValidationReport};

/// Struct Validator chịu trách nhiệm thực thi toàn bộ luồng kiểm tra Policy
pub struct PolicyValidator;

impl PolicyValidator {
    /// Đánh giá toàn bộ Policy và sinh ra ValidationReport
    pub fn validate(policy: &FirewallPolicy) -> ValidationReport {
        let mut report = ValidationReport {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            infos: Vec::new(),
        };

        // 1. Kiểm tra Schema Version & Kind
        if let Err(e) = policy.validate_schema_version() {
            report.add_issue(ValidationIssue::error(
                "INVALID_SCHEMA_VERSION",
                e.user_message(),
                Some("apiVersion/kind".to_string()),
                None,
            ));
            return report;
        }

        // 2. Schema Validation từng Rule
        Self::validate_schema(policy, &mut report);

        // 3. Semantic & Security Validation
        Self::validate_semantics(policy, &mut report);

        report
    }

    /// Kiểm tra tính đúng đắn của Schema và định dạng dữ liệu
    fn validate_schema(policy: &FirewallPolicy, report: &mut ValidationReport) {
        let mut seen_rule_ids = HashSet::new();

        for (idx, rule) in policy.rules.iter().enumerate() {
            let field_prefix = format!("rules[{idx}]");

            // Kiểm tra Rule ID rỗng
            if rule.id.as_str().trim().is_empty() {
                report.add_issue(ValidationIssue::error(
                    "EMPTY_RULE_ID",
                    "Rule ID must not be empty",
                    Some(format!("{field_prefix}.id")),
                    Some(rule.id.clone()),
                ));
            } else if !seen_rule_ids.insert(rule.id.clone()) {
                // Kiểm tra Rule ID trùng lặp
                report.add_issue(ValidationIssue::error(
                    "DUPLICATE_RULE_ID",
                    format!("Duplicate rule ID detected: '{}'", rule.id),
                    Some(format!("{field_prefix}.id")),
                    Some(rule.id.clone()),
                ));
            }

            // Kiểm tra PortSpec trong rule
            for port_spec in rule
                .source_ports
                .iter()
                .chain(rule.destination_ports.iter())
            {
                if let Err(e) = port_spec.validate() {
                    report.add_issue(ValidationIssue::error(
                        "INVALID_PORT_SPEC",
                        e.user_message(),
                        Some(format!("{field_prefix}.ports")),
                        Some(rule.id.clone()),
                    ));
                }
            }

            // Kiểm tra CidrSpec trong rule
            for cidr_spec in rule
                .source_cidrs
                .iter()
                .chain(rule.destination_cidrs.iter())
            {
                if let Err(e) = cidr_spec.validate() {
                    report.add_issue(ValidationIssue::error(
                        "INVALID_CIDR_SPEC",
                        e.user_message(),
                        Some(format!("{field_prefix}.cidrs")),
                        Some(rule.id.clone()),
                    ));
                }
            }

            // Kiểm tra tương thích giữa Protocol và Ports (VD: ICMP không được đi kèm destination_ports)
            if let Some(protocol) = rule.protocol {
                if (protocol == TransportProtocol::Icmp || protocol == TransportProtocol::Icmpv6)
                    && (!rule.source_ports.is_empty() || !rule.destination_ports.is_empty())
                {
                    report.add_issue(ValidationIssue::error(
                        "INCOMPATIBLE_PROTOCOL_PORTS",
                        format!("Protocol '{protocol:?}' cannot be specified with ports"),
                        Some(format!("{field_prefix}.protocol")),
                        Some(rule.id.clone()),
                    ));
                }
            }

            // Kiểm tra RateLimit > 0
            if let Some(rate_limit) = &rule.rate_limit {
                if rate_limit.packets_per_second == 0 || rate_limit.burst == 0 {
                    report.add_issue(ValidationIssue::error(
                        "INVALID_RATE_LIMIT",
                        "Rate limit packets_per_second and burst must be greater than 0",
                        Some(format!("{field_prefix}.rateLimit")),
                        Some(rule.id.clone()),
                    ));
                }
            }

            // Kiểm tra tên Interface không rỗng
            for interface in &rule.interfaces {
                if let InterfaceSelector::Name(name) = interface {
                    if name.trim().is_empty() {
                        report.add_issue(ValidationIssue::error(
                            "EMPTY_INTERFACE_NAME",
                            "Interface name must not be empty",
                            Some(format!("{field_prefix}.interfaces")),
                            Some(rule.id.clone()),
                        ));
                    }
                }
            }
        }
    }

    /// Kiểm tra Semantic logic và các rủi ro bảo mật an toàn mạng
    fn validate_semantics(policy: &FirewallPolicy, report: &mut ValidationReport) {
        // Cảnh báo nếu Policy không có bất kỳ rule nào kiểm soát Output
        if policy.defaults.output == FirewallAction::Accept
            && policy
                .rules
                .iter()
                .all(|r| r.direction != FirewallDirection::Output)
        {
            report.add_issue(ValidationIssue::info(
                "UNRESTRICTED_OUTPUT",
                "Policy output default is ACCEPT with no specific output rules.",
                Some("defaults.output".to_string()),
                None,
            ));
        }

        // Cảnh báo nếu Default Input là DROP nhưng thiếu allow Loopback interface (`lo`)
        if policy.defaults.input == FirewallAction::Drop {
            let has_loopback = policy.rules.iter().any(|r| {
                r.direction == FirewallDirection::Input
                    && r.action == FirewallAction::Accept
                    && r.interfaces.iter().any(|i| match i {
                        InterfaceSelector::Name(name) => name == "lo" || name == "loopback",
                        _ => false,
                    })
            });

            if !has_loopback {
                report.add_issue(ValidationIssue::warning(
                    "MISSING_LOOPBACK_ALLOW",
                    "Default input drop specified but no explicit allow rule for loopback interface (lo). Local communications may break.",
                    Some("defaults.input".to_string()),
                    None,
                ));
            }

            // Cảnh báo nếu Default Input là DROP nhưng thiếu allow `established` / `related` connection states
            let has_established = policy.rules.iter().any(|r| {
                r.direction == FirewallDirection::Input
                    && r.action == FirewallAction::Accept
                    && (r.connection_states.contains(&ConnectionState::Established)
                        || r.connection_states.contains(&ConnectionState::Related))
            });

            if !has_established {
                report.add_issue(ValidationIssue::warning(
                    "MISSING_ESTABLISHED_ALLOW",
                    "Default input drop specified but no rule allowing ESTABLISHED/RELATED connection states. Outbound responses will be dropped.",
                    Some("defaults.input".to_string()),
                    None,
                ));
            }
        }

        // Cảnh báo an ninh mở cổng nhạy cảm (SSH/Databases) ra toàn Internet (0.0.0.0/0 hoặc ::/0)
        for rule in &policy.rules {
            if rule.direction == FirewallDirection::Input && rule.action == FirewallAction::Accept {
                let is_open_to_world = rule.source_cidrs.is_empty()
                    || rule
                        .source_cidrs
                        .iter()
                        .any(|c| c.0 == "0.0.0.0/0" || c.0 == "::/0");

                if is_open_to_world {
                    for port in &rule.destination_ports {
                        match port {
                            PortSpec::Single(22) => {
                                report.add_issue(ValidationIssue::warning(
                                    "SSH_EXPOSED_INTERNET",
                                    "SSH port 22 is allowed from any source IP (0.0.0.0/0). Consider restricting SSH source CIDRs.",
                                    Some("rules.destinationPorts".to_string()),
                                    Some(rule.id.clone()),
                                ));
                            }
                            PortSpec::Single(5432) => {
                                report.add_issue(ValidationIssue::warning(
                                    "DATABASE_EXPOSED_WAN",
                                    "PostgreSQL database port 5432 is allowed from any source IP. Highly recommended to restrict access.",
                                    Some("rules.destinationPorts".to_string()),
                                    Some(rule.id.clone()),
                                ));
                            }
                            PortSpec::Single(6379) => {
                                report.add_issue(ValidationIssue::warning(
                                    "DATABASE_EXPOSED_WAN",
                                    "Redis in-memory database port 6379 is allowed from any source IP. High risk of unauthorized data access.",
                                    Some("rules.destinationPorts".to_string()),
                                    Some(rule.id.clone()),
                                ));
                            }
                            PortSpec::Single(3306) => {
                                report.add_issue(ValidationIssue::warning(
                                    "DATABASE_EXPOSED_WAN",
                                    "MySQL database port 3306 is allowed from any source IP. Consider restricting source CIDRs.",
                                    Some("rules.destinationPorts".to_string()),
                                    Some(rule.id.clone()),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Phát hiện trùng lặp hoặc xung đột giữa các rules
        for i in 0..policy.rules.len() {
            for j in (i + 1)..policy.rules.len() {
                let r1 = &policy.rules[i];
                let r2 = &policy.rules[j];

                // Nếu hai rules giống hệt nhau về điều kiện matching
                if r1.direction == r2.direction
                    && r1.protocol == r2.protocol
                    && r1.destination_ports == r2.destination_ports
                    && r1.source_cidrs == r2.source_cidrs
                    && r1.interfaces == r2.interfaces
                {
                    if r1.action == r2.action {
                        report.add_issue(ValidationIssue::warning(
                            "SHADOWED_RULE",
                            format!(
                                "Rule '{}' is shadowed by identical preceding Rule '{}'",
                                r2.id, r1.id
                            ),
                            Some(format!("rules[{j}]")),
                            Some(r2.id.clone()),
                        ));
                    } else {
                        report.add_issue(ValidationIssue::error(
                            "CONFLICTING_RULES",
                            format!("Rule '{}' conflicts with preceding Rule '{}' (Action {:?} vs {:?})", r2.id, r1.id, r2.action, r1.action),
                            Some(format!("rules[{j}]")),
                            Some(r2.id.clone()),
                        ));
                    }
                }
            }
        }
    }
}
