// Integration Test cho Advanced Networking (Phase 24 Enterprise Bonding, VRF & SYN Flood)

use aegis_firewall::AdvancedNetworkManager;
use aegis_models::advanced_network::{SynProxyConfig, VrfProfile};

#[test]
fn test_management_interface_protection() {
    let mgmt_iface = "eth0";
    let valid_slaves = vec!["eth1".to_string(), "eth2".to_string()];

    // 1. Slaves không chứa eth0 -> Hợp lệ
    assert!(
        AdvancedNetworkManager::validate_management_interface_protection(&valid_slaves, mgmt_iface)
            .is_ok()
    );

    // 2. Slaves chứa eth0 -> Phải bị từ chối để tránh ngắt kết nối SSH
    let invalid_slaves = vec!["eth0".to_string(), "eth1".to_string()];
    let result = AdvancedNetworkManager::validate_management_interface_protection(
        &invalid_slaves,
        mgmt_iface,
    );

    assert!(
        result.is_err(),
        "Thêm card mạng Quản trị SSH (eth0) vào bond slave phải bị từ chối với lỗi Validation"
    );
}

#[test]
fn test_vrf_profile_validation() {
    // 1. VRF hợp lệ
    let valid_vrf = VrfProfile {
        vrf_name: "vrf-prod".to_string(),
        table_id: 100,
        interfaces: vec!["eth1".to_string()],
    };
    assert!(AdvancedNetworkManager::validate_vrf_profile(&valid_vrf).is_ok());

    // 2. VRF thiếu tên -> Thất bại
    let invalid_name_vrf = VrfProfile {
        vrf_name: "".to_string(),
        table_id: 100,
        interfaces: vec![],
    };
    assert!(AdvancedNetworkManager::validate_vrf_profile(&invalid_name_vrf).is_err());

    // 3. VRF table_id không hợp lệ (table_id = 0) -> Thất bại
    let invalid_table_vrf = VrfProfile {
        vrf_name: "vrf-test".to_string(),
        table_id: 0,
        interfaces: vec![],
    };
    assert!(AdvancedNetworkManager::validate_vrf_profile(&invalid_table_vrf).is_err());
}

#[test]
fn test_synproxy_rule_generation() {
    let syn_config = SynProxyConfig {
        mss: 1460,
        wscale: 7,
        syn_rate_limit: 100,
    };

    let nft_rule = AdvancedNetworkManager::generate_synproxy_nft_rule(&syn_config);

    assert!(nft_rule.contains("synproxy mss 1460 wscale 7"));
    assert!(nft_rule.contains("limit rate 100/second burst 200 packets accept"));
}

#[test]
fn test_dynamic_ip_set_update_commands() {
    let ips = vec!["1.2.3.4".to_string(), "5.6.7.8".to_string()];

    // 1. Generates add command
    let add_cmd = AdvancedNetworkManager::generate_dynamic_set_add_command("blocklist", &ips);
    assert_eq!(
        add_cmd,
        "nft add element inet aegisnode blocklist { 1.2.3.4, 5.6.7.8 }"
    );

    // 2. Generates delete command
    let del_cmd = AdvancedNetworkManager::generate_dynamic_set_delete_command("blocklist", &ips);
    assert_eq!(
        del_cmd,
        "nft delete element inet aegisnode blocklist { 1.2.3.4, 5.6.7.8 }"
    );
}
