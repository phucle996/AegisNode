// Integration / Unit tests cho Policy Validator, Normalizer và Hasher

use aegis_models::firewall::FirewallPolicy;
use aegis_policy::{PolicyHasher, PolicyValidator};

#[test]
fn test_validate_minimal_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/minimal.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let report = PolicyValidator::validate(&policy);
    assert!(report.is_valid());
    assert!(report.errors.is_empty());
}

#[test]
fn test_validate_web_server_policy_security_warnings() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let report = PolicyValidator::validate(&policy);
    // Policy vẫn valid (không có ERROR) nhưng có thể chứa cảnh báo an ninh
    assert!(report.is_valid());
}

#[test]
fn test_validate_conflicting_rules() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/conflicting-rules.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let report = PolicyValidator::validate(&policy);
    assert!(!report.is_valid());
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.code == "CONFLICTING_RULES" || e.code == "DUPLICATE_RULE_ID")
    );
}

#[test]
fn test_validate_missing_loopback_warning() {
    let yaml_str = r#"
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
metadata:
  name: no-loopback-policy
defaults:
  input: drop
  output: accept
  forward: drop
rules:
  - id: allow-http
    direction: input
    action: accept
    protocol: tcp
    destinationPorts: [80]
"#;
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();
    let report = PolicyValidator::validate(&policy);

    assert!(report.is_valid());
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.code == "MISSING_LOOPBACK_ALLOW")
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.code == "MISSING_ESTABLISHED_ALLOW")
    );
}

#[test]
fn test_validate_ssh_exposed_warning() {
    let yaml_str = r#"
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
metadata:
  name: ssh-open-policy
defaults:
  input: drop
  output: accept
  forward: drop
rules:
  - id: allow-ssh
    direction: input
    action: accept
    protocol: tcp
    destinationPorts: [22]
    sourceCidrs: ["0.0.0.0/0"]
"#;
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();
    let report = PolicyValidator::validate(&policy);

    assert!(report.is_valid());
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.code == "SSH_EXPOSED_INTERNET")
    );
}

#[test]
fn test_validate_database_exposed_warning() {
    let yaml_str = r#"
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
metadata:
  name: postgres-open-policy
defaults:
  input: drop
  output: accept
  forward: drop
rules:
  - id: allow-postgres
    direction: input
    action: accept
    protocol: tcp
    destinationPorts: [5432]
    sourceCidrs: ["0.0.0.0/0"]
"#;
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();
    let report = PolicyValidator::validate(&policy);

    assert!(report.is_valid());
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.code == "DATABASE_EXPOSED_WAN")
    );
}

#[test]
fn test_deterministic_policy_hash() {
    let yaml1 = r#"
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
metadata:
  name: test-hash
  id: policy-hash-001
defaults:
  input: drop
  output: accept
  forward: drop
rules:
  - id: rule-1
    direction: input
    action: accept
    sourceCidrs: ["10.0.0.0/8", "192.168.1.0/24"]
    destinationPorts: [443, 80]
"#;

    let yaml2 = r#"
apiVersion: aegisnode.io/v1
kind: FirewallPolicy
metadata:
  name: test-hash
  id: policy-hash-001
defaults:
  input: drop
  output: accept
  forward: drop
rules:
  - id: rule-1
    direction: input
    action: accept
    sourceCidrs: ["192.168.1.0/24", "10.0.0.0/8"]
    destinationPorts: [80, 443]
"#;

    let policy1: FirewallPolicy = serde_yaml::from_str(yaml1).unwrap();
    let policy2: FirewallPolicy = serde_yaml::from_str(yaml2).unwrap();

    let hash1 = PolicyHasher::compute_hash(&policy1);
    let hash2 = PolicyHasher::compute_hash(&policy2);

    assert_eq!(
        hash1, hash2,
        "Deterministic Policy Hasher must produce identical hashes for equivalent policies"
    );
}
