// Normalizer chuẩn hóa cấu trúc Firewall Policy (Sort CIDR, sort/dedup Ports, normalize metadata timestamp)
// Đảm bảo dữ liệu Policy ở dạng chuẩn để phục vụ việc sinh Policy Hash deterministic

use aegis_models::firewall::{CidrSpec, FirewallPolicy, PortSpec};
use chrono::DateTime;

/// Struct Normalizer chịu trách nhiệm chuẩn hóa cấu trúc dữ liệu Policy
pub struct PolicyNormalizer;

impl PolicyNormalizer {
    /// Đưa FirewallPolicy về dạng chuẩn (Deterministic Canonical Form)
    pub fn normalize(policy: &FirewallPolicy) -> FirewallPolicy {
        let mut normalized = policy.clone();

        // 1. Chuẩn hóa metadata timestamp về UNIX_EPOCH để thời gian parse không ảnh hưởng hash nội dung
        normalized.metadata.created_at = DateTime::UNIX_EPOCH;

        // 2. Chuẩn hóa từng Rule trong Policy
        for rule in &mut normalized.rules {
            // Sắp xếp & loại bỏ duplicate Source CIDRs
            rule.source_cidrs = Self::normalize_cidrs(&rule.source_cidrs);

            // Sắp xếp & loại bỏ duplicate Destination CIDRs
            rule.destination_cidrs = Self::normalize_cidrs(&rule.destination_cidrs);

            // Sắp xếp & loại bỏ duplicate Source Ports
            rule.source_ports = Self::normalize_ports(&rule.source_ports);

            // Sắp xếp & loại bỏ duplicate Destination Ports
            rule.destination_ports = Self::normalize_ports(&rule.destination_ports);
        }

        normalized
    }

    /// Sắp xếp và loại bỏ CIDRs trùng lặp
    fn normalize_cidrs(cidrs: &[CidrSpec]) -> Vec<CidrSpec> {
        let mut list: Vec<CidrSpec> = cidrs.to_vec();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list.dedup();
        list
    }

    /// Sắp xếp và loại bỏ Ports trùng lặp
    fn normalize_ports(ports: &[PortSpec]) -> Vec<PortSpec> {
        let mut list: Vec<PortSpec> = ports.to_vec();
        list.sort_by(|a, b| match (a, b) {
            (PortSpec::Single(p1), PortSpec::Single(p2)) => p1.cmp(p2),
            (PortSpec::Single(p1), PortSpec::Range(r1, _)) => p1.cmp(r1),
            (PortSpec::Range(r1, _), PortSpec::Single(p2)) => r1.cmp(p2),
            (PortSpec::Range(r1_s, r1_e), PortSpec::Range(r2_s, r2_e)) => {
                r1_s.cmp(r2_s).then_with(|| r1_e.cmp(r2_e))
            }
        });
        list.dedup();
        list
    }
}
