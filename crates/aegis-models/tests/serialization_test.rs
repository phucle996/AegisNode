// Integration / Unit tests cho Serialization, Deserialization & Validation trong crate aegis-models

use aegis_models::firewall::{CidrSpec, FirewallPolicy, PortSpec, SUPPORTED_API_VERSION};

#[test]
fn test_parse_minimal_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/minimal.yaml");
    let policy: FirewallPolicy =
        serde_yaml::from_str(yaml_str).expect("Failed to parse minimal.yaml");

    assert_eq!(policy.api_version, SUPPORTED_API_VERSION);
    assert_eq!(policy.metadata.name, "minimal-policy");
    assert_eq!(policy.rules.len(), 1);
    assert!(policy.validate_schema_version().is_ok());
}

#[test]
fn test_parse_web_server_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy =
        serde_yaml::from_str(yaml_str).expect("Failed to parse web-server.yaml");

    assert_eq!(policy.metadata.name, "web-server-policy");
    assert_eq!(policy.rules.len(), 4);

    // Kiểm tra PortSpec cho HTTP/HTTPS ports (80, 443)
    let http_rule = &policy.rules[2];
    assert_eq!(http_rule.destination_ports.len(), 2);
    assert_eq!(http_rule.destination_ports[0], PortSpec::Single(80));
    assert_eq!(http_rule.destination_ports[1], PortSpec::Single(443));
}

#[test]
fn test_parse_docker_host_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/docker-host.yaml");
    let policy: FirewallPolicy =
        serde_yaml::from_str(yaml_str).expect("Failed to parse docker-host.yaml");

    assert_eq!(policy.metadata.name, "docker-host-policy");
    let port_rule = &policy.rules[2];
    assert_eq!(port_rule.destination_ports[0], PortSpec::Range(8080, 8090));
}

#[test]
fn test_reject_invalid_port() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/invalid-port.yaml");
    let result: Result<FirewallPolicy, _> = serde_yaml::from_str(yaml_str);
    assert!(
        result.is_err(),
        "Expected error when parsing port 70000 > 65535"
    );
}

#[test]
fn test_reject_invalid_cidr() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/invalid-cidr.yaml");
    let result: Result<FirewallPolicy, _> = serde_yaml::from_str(yaml_str);
    assert!(result.is_err(), "Expected error when parsing prefix /35");
}

#[test]
fn test_port_spec_validation() {
    assert!(PortSpec::Single(80).validate().is_ok());
    assert!(PortSpec::Single(0).validate().is_err());
    assert!(PortSpec::Range(80, 8080).validate().is_ok());
    assert!(PortSpec::Range(8080, 80).validate().is_err());
}

#[test]
fn test_cidr_spec_validation() {
    assert!(CidrSpec("192.168.1.0/24".to_string()).validate().is_ok());
    assert!(CidrSpec("10.0.0.1/32".to_string()).validate().is_ok());
    assert!(CidrSpec("2001:db8::/32".to_string()).validate().is_ok());
    assert!(CidrSpec("192.168.1.1/33".to_string()).validate().is_err());
    assert!(CidrSpec("invalid-ip/24".to_string()).validate().is_err());
}

#[test]
fn test_roundtrip_serialization() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    // Round-trip YAML -> Struct -> JSON -> Struct -> YAML
    let json_bytes = serde_json::to_string_pretty(&policy).unwrap();
    let policy_from_json: FirewallPolicy = serde_json::from_str(&json_bytes).unwrap();

    assert_eq!(policy, policy_from_json);

    let reserialized_yaml = serde_yaml::to_string(&policy_from_json).unwrap();
    let policy_from_reserialized: FirewallPolicy =
        serde_yaml::from_str(&reserialized_yaml).unwrap();

    assert_eq!(policy, policy_from_reserialized);
}
