// Integration / Golden tests cho NftablesCompiler và NatCompiler trong aegis-firewall

use aegis_firewall::{CompiledFirewallPolicy, FirewallCompiler, NatCompiler, NftablesCompiler};
use aegis_models::firewall::FirewallPolicy;
use aegis_models::nat::NatPolicy;
use std::process::Command;

#[test]
fn test_compile_web_server_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/web-server.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let compiler = NftablesCompiler::new();
    let compiled: CompiledFirewallPolicy = compiler
        .compile(&policy)
        .expect("Failed to compile web-server.yaml");

    assert!(compiled.nft_script.contains("table inet aegis_filter"));
    assert!(
        compiled
            .nft_script
            .contains("destroy table inet aegis_filter")
    );
    assert!(!compiled.nft_script.contains("flush ruleset")); // KHÔNG ĐƯỢC CHỨA flush ruleset
    assert!(compiled.nft_script.contains("aegis:rule:allow-http-https"));
    assert!(compiled.nft_script.contains("tcp dport { 80, 443 }"));
    assert!(compiled.nft_script.contains("counter"));

    // Kiểm tra syntax bằng binary nft nếu có sẵn trên môi trường Linux và đủ quyền
    verify_nft_syntax(&compiled.nft_script);
}

#[test]
fn test_compile_docker_host_policy() {
    let yaml_str = include_str!("../../../tests/fixtures/policies/docker-host.yaml");
    let policy: FirewallPolicy = serde_yaml::from_str(yaml_str).unwrap();

    let compiler = NftablesCompiler::new();
    let compiled = compiler
        .compile(&policy)
        .expect("Failed to compile docker-host.yaml");

    assert!(compiled.nft_script.contains("tcp dport 8080-8090"));
    assert!(compiled.nft_script.contains("iifname \"docker0\""));
    assert!(compiled.nft_script.contains("counter"));

    verify_nft_syntax(&compiled.nft_script);
}

#[test]
fn test_compile_router_nat_policy() {
    let nat_yaml = r#"
apiVersion: aegisnode.io/v1
kind: NatPolicy
metadata:
  name: router-nat
masqueradeRules:
  - id: masq-lan-wan
    outInterface: eth0
    sourceCidr: 192.168.1.0/24
portForwardRules:
  - id: pf-web
    inInterface: eth0
    protocol: tcp
    externalPort: 8080
    destinationAddress: 192.168.1.100
    destinationPort: 80
"#;
    let nat_policy: NatPolicy = serde_yaml::from_str(nat_yaml).unwrap();
    let nat_compiler = NatCompiler::new();
    let script = nat_compiler
        .compile(&nat_policy)
        .expect("Failed to compile NAT policy");

    assert!(script.contains("table ip aegis_nat"));
    assert!(script.contains("destroy table ip aegis_nat"));
    assert!(script.contains("masquerade"));
    assert!(script.contains("dnat to 192.168.1.100:80"));
    assert!(script.contains("aegis:nat:masq-lan-wan"));

    verify_nft_syntax(&script);
}

/// Thử nghiệm kiểm tra cú pháp trực tiếp bằng `nft --check --file` nếu binary nft khả dụng và có quyền netlink
fn verify_nft_syntax(script: &str) {
    if let Ok(output) = Command::new("nft").arg("--version").output() {
        if output.status.success() {
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!("aegis_test_{}.nft", uuid::Uuid::new_v4()));
            std::fs::write(&temp_file, script).expect("Failed to write temp nft file");

            let check_res = Command::new("nft")
                .arg("--check")
                .arg("--file")
                .arg(&temp_file)
                .output();

            let _ = std::fs::remove_file(&temp_file);

            if let Ok(check_output) = check_res {
                if !check_output.status.success() {
                    let stderr = String::from_utf8_lossy(&check_output.stderr);
                    // Bỏ qua lỗi Operation not permitted khi chạy unit test trong môi trường unprivileged (thiếu CAP_NET_ADMIN netlink socket)
                    if stderr.contains("Operation not permitted")
                        || stderr.contains("Permission denied")
                    {
                        return;
                    }
                    panic!(
                        "nft --check failed for script:\n{}\nError:\n{}",
                        script, stderr
                    );
                }
            }
        }
    }
}
